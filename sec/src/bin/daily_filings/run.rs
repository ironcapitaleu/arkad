//! # Run
//!
//! Provides [`run`], which runs the machines of each date of a `daily_filings` run, the outcome of
//! each date, the rules that sort an error into an outcome, and the [`RunSummary`] of the run.
//!
//! A date is pending if its daily index does not exist yet and a later run reads the date again.
//! A filing is unextractable if its folder holds no XBRL instance. Neither fails the run.

use std::fmt::{self, Display, Formatter};
use std::future::Future;
use std::time::Duration;

use chrono::{Days, NaiveDate};
use futures_util::StreamExt;
use sec::error::{State as StateError, StateMachine};
use sec::implementations::states::extract::daily_index::{
    DetectFilingsSuperState, DetectedFilings,
};
use sec::implementations::states::extract::filing::ExtractFilingSuperState;
use sec::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
use sec::implementations::states::extract::submissions::ExtractSubmissionsSuperState;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::implementations::states::extract::validate_index_date::ValidateIndexDate;
use sec::prelude::{IntoStateMachineStream, StateMachineStream, StreamError, StreamItem};
use sec::shared::cik::Cik;
use sec::shared::filing_documents::FilingDocumentsError;
use sec::shared::filing_ref::FilingRef;
use sec::shared::financial::accession_number::AccessionNumber;
use sec::shared::http_client::Executes;
use sec::shared::http_client::SecClient as SecClientTrait;
use sec::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::{RawDailyIndex, RawSubmissions, RawXbrlInstance, RawXbrlLabels};
use sec::shared::request::implementations::sec_request::{FilingDocumentSet, FilingIndex};
use sec::shared::response::implementations::sec_response::{JsonBody, SecResponse, ZipBody};
use serde_json::{Value, json};
use storage::implementations::repository::filesystem::{
    FilesystemRepository, IngestionMode, InputDocument, Write,
};
use thiserror::Error;
use uuid::Uuid;

use crate::constants::{IN_FLIGHT, PENDING_DAYS};

/// The SEC client that every machine of a run accepts.
// Deviation: the bound names `SecClientTrait` once per response type, because Rust 1.90 rejects
// two bindings of `Response` in one bound (E0719).
pub trait DailyClient:
    SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>
    + SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>>
    + Executes<FilingIndex>
    + Executes<FilingDocumentSet>
    + 'static
{
}

impl<C> DailyClient for C where
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>
        + SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>>
        + Executes<FilingIndex>
        + Executes<FilingDocumentSet>
        + 'static
{
}

type DailyIndexRepository = FilesystemRepository<Write, RawDocument<RawDailyIndex>>;
type InstanceRepository = FilesystemRepository<Write, RawDocument<RawXbrlInstance>>;
type LabelsRepository = FilesystemRepository<Write, RawDocument<RawXbrlLabels>>;
type SubmissionsRepository = FilesystemRepository<Write, RawDocument<RawSubmissions>>;

/// What an error that `run` reports means for the run, so the caller can pick the log level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// The SEC answered 503, and the machine runs again.
    Retried,
    /// The daily index does not exist yet, and a later run reads the date again.
    Pending,
    /// The folder of the filing holds no XBRL instance.
    Unextractable,
    /// The error fails the run.
    Failed,
}

impl Display for ErrorKind {
    /// Writes the kind in lowercase, as the run summary writes its outcomes.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Retried => "retried",
            Self::Pending => "pending",
            Self::Unextractable => "unextractable",
            Self::Failed => "failed",
        })
    }
}

/// The outcome of one date.
#[derive(Debug)]
pub enum DateOutcome {
    /// Detection returned the selected filings, and the run tried to extract each one and its
    /// company.
    Processed {
        /// The outcome of each selected filing.
        filings: Vec<FilingOutcome>,
        /// The outcome of the `submissions` file of each company.
        companies: Vec<CompanyOutcome>,
    },
    /// The daily index does not exist yet, and a later run reads the date again.
    Pending(StreamError),
    /// Detection failed.
    Failed(StreamError),
}

/// The outcome of one filing: stored, or the error of the state that failed.
#[derive(Debug)]
pub struct FilingOutcome {
    /// The accession number of the filing.
    pub accession_number: AccessionNumber,
    /// `Ok` if the run stored the documents of the filing.
    pub result: Result<(), StreamError>,
}

/// The outcome of the `submissions` file of one company.
#[derive(Debug)]
pub struct CompanyOutcome {
    /// The CIK of the company.
    pub cik: Cik,
    /// `Ok` if the run stored the `submissions` file.
    pub result: Result<(), StreamError>,
}

/// The outcome of every date of one run, oldest date first.
#[derive(Debug)]
pub struct RunSummary(pub Vec<(NaiveDate, DateOutcome)>);

/// The error of a run in which at least one date, filing, or `submissions` file failed.
#[derive(Debug, Error, PartialEq, Eq)]
#[error(
    "[FailedDailyRun] A run of {dates} dates has {failures} failed dates, filings, or submissions files"
)]
pub struct FailedDailyRun {
    /// The number of failed dates, filings, and `submissions` files.
    pub failures: usize,
    /// The number of dates in the run.
    pub dates: usize,
}

/// Returns `true` if the detection of `date` failed only because its daily index does not exist
/// yet, and a later run reads `date` again.
///
/// That holds if `error` is a `MissingDailyIndex`, `date` is not `first_date`, and `date` is one
/// of the [`PENDING_DAYS`] days that end with `yesterday`. The first date of a run can be its last
/// read, for example `yesterday-3` in the scheduled run, so a missing index there fails the run.
/// This keeps the rule safe whatever window the job sets.
pub fn is_pending(
    error: &StreamError,
    date: NaiveDate,
    first_date: NaiveDate,
    yesterday: NaiveDate,
) -> bool {
    let oldest_pending = yesterday
        .checked_sub_days(Days::new(PENDING_DAYS - 1))
        .expect("Given a date of the current era, a date a few days before it always exists");
    date != first_date
        && date >= oldest_pending
        && matches!(
            &error.source,
            StateMachine::State(StateError::MissingDailyIndex(_))
        )
}

/// Returns `true` if the folder of the filing holds no XBRL instance.
///
/// A filing like that fails the same way on every attempt, so it does not fail the run. A folder
/// with more than one candidate fails the run, because it shows a gap in the selection rules.
pub const fn is_unextractable(error: &StreamError) -> bool {
    matches!(
        &error.source,
        StateMachine::State(StateError::FailedDocumentSelection(failure))
            if matches!(
                failure.filing_documents_error,
                FilingDocumentsError::MissingXbrlInstance { .. }
            )
    )
}

impl RunSummary {
    /// Returns `Ok` if no date, filing, or `submissions` file failed.
    ///
    /// # Errors
    ///
    /// Returns [`FailedDailyRun`] if at least one of them failed. A pending date and an
    /// unextractable filing do not count as failed.
    pub fn result(&self) -> Result<(), FailedDailyRun> {
        match self.failure_count() {
            0 => Ok(()),
            failures => Err(FailedDailyRun {
                failures,
                dates: self.0.len(),
            }),
        }
    }

    /// Returns the number of failed dates, filings, and `submissions` files.
    fn failure_count(&self) -> usize {
        self.0
            .iter()
            .map(|(_, outcome)| match outcome {
                DateOutcome::Processed { filings, companies } => {
                    let failed_filings = filings
                        .iter()
                        .filter(|filing| {
                            filing
                                .result
                                .as_ref()
                                .is_err_and(|error| !is_unextractable(error))
                        })
                        .count();
                    let failed_companies = companies
                        .iter()
                        .filter(|company| company.result.is_err())
                        .count();
                    failed_filings + failed_companies
                }
                DateOutcome::Pending(_) => 0,
                DateOutcome::Failed(_) => 1,
            })
            .sum()
    }

    /// Returns one JSON entry per date.
    ///
    /// Each entry names the date and the outcome of detection: `processed`, `empty` (no filing
    /// selected), `pending`, or `failed`. A processed date lists the counts of selected and stored
    /// filings and `submissions` files, and each unextractable or failed one with its error.
    pub fn entries(&self) -> Vec<Value> {
        self.0
            .iter()
            .map(|(date, outcome)| match outcome {
                DateOutcome::Processed { filings, companies } => json!({
                    "date": date.to_string(),
                    "detection": if filings.is_empty() { "empty" } else { "processed" },
                    "filings": filing_entries(filings),
                    "submissions": company_entries(companies),
                }),
                DateOutcome::Pending(error) => json!({
                    "date": date.to_string(),
                    "detection": "pending",
                    "error": error.to_string(),
                }),
                DateOutcome::Failed(error) => json!({
                    "date": date.to_string(),
                    "detection": "failed",
                    "failed_state": error.state_name,
                    "error": error.to_string(),
                }),
            })
            .collect()
    }
}

/// Returns the counts of the filings of one date, and each unextractable or failed filing.
fn filing_entries(filings: &[FilingOutcome]) -> Value {
    let errors = |unextractable: bool| -> Vec<Value> {
        filings
            .iter()
            .filter_map(|filing| filing.result.as_ref().err().map(|error| (filing, error)))
            .filter(|(_, error)| is_unextractable(error) == unextractable)
            .map(|(filing, error)| {
                json!({
                    "accession_number": filing.accession_number.to_string(),
                    "failed_state": error.state_name,
                    "error": error.to_string(),
                })
            })
            .collect()
    };
    json!({
        "selected": filings.len(),
        "stored": filings.iter().filter(|filing| filing.result.is_ok()).count(),
        "unextractable": errors(true),
        "failed": errors(false),
    })
}

/// Returns the counts of the `submissions` files of one date, and each failed one.
fn company_entries(companies: &[CompanyOutcome]) -> Value {
    let failed: Vec<Value> = companies
        .iter()
        .filter_map(|company| {
            company.result.as_ref().err().map(|error| {
                json!({
                    "cik": company.cik.to_string(),
                    "failed_state": error.state_name,
                    "error": error.to_string(),
                })
            })
        })
        .collect();
    json!({
        "companies": companies.len(),
        "stored": companies.iter().filter(|company| company.result.is_ok()).count(),
        "failed": failed,
    })
}

/// The stores that one run writes to.
#[derive(Debug, Clone)]
pub struct Repositories {
    /// The store of the daily indexes.
    pub daily_index: DailyIndexRepository,
    /// The store of the XBRL instances.
    pub instance: InstanceRepository,
    /// The store of the label files.
    pub labels: LabelsRepository,
    /// The store of the `submissions` files.
    pub submissions: SubmissionsRepository,
}

/// The fixed inputs of one run.
#[derive(Debug, Clone)]
pub struct Settings {
    /// The ID that every machine of the run carries.
    pub execution_id: Uuid,
    /// Yesterday in New York, for the pending rule.
    pub yesterday: NaiveDate,
    /// The waits before each new attempt of a machine that the SEC answered with 503.
    pub retry_delays: Vec<Duration>,
}

/// Runs the new-filing path for each date of `dates`, in order, and returns the summary.
///
/// `dates` must be sorted oldest first, as `run_dates` returns them, because the pending rule
/// never applies to the first date.
///
/// For each date, it runs detection, then one filing machine per selected filing, then one
/// `submissions` machine per distinct CIK of the selected filings. At most [`IN_FLIGHT`] filing
/// or `submissions` machines run at once. A filing or `submissions` machine that the SEC answers
/// with 503 runs again after each wait of [`Settings::retry_delays`]. A failure does not stop the
/// other machines or the other dates.
///
/// Calls `on_item` with a JSON context and each successful stream item, and `on_error` with a
/// JSON context, each error, and its [`ErrorKind`]. The context holds the date, and the accession number or the CIK.
pub async fn run<C>(
    dates: &[NaiveDate],
    sec_client: &C,
    repositories: &Repositories,
    settings: &Settings,
    on_item: impl Fn(&Value, &StreamItem) + Send + Sync,
    on_error: impl Fn(&Value, &StreamError, ErrorKind) + Send + Sync,
) -> RunSummary
where
    C: DailyClient,
    DetectFilingsSuperState<ValidateIndexDate, DailyIndexRepository, C>: IntoStateMachineStream,
    ExtractFilingSuperState<
        PrepareSecRequest<FilingIndex>,
        InstanceRepository,
        LabelsRepository,
        C,
    >: IntoStateMachineStream,
    ExtractSubmissionsSuperState<ValidateCikFormat, SubmissionsRepository, C>:
        IntoStateMachineStream,
{
    let Some(&first_date) = dates.first() else {
        return RunSummary(Vec::new());
    };
    let mut outcomes = Vec::with_capacity(dates.len());
    for &date in dates {
        let context = json!({ "date": date.to_string() });
        let detection = DetectFilingsSuperState::builder()
            .date(date.to_string())
            .sec_client(sec_client.clone())
            .execution_id(settings.execution_id)
            .repository(repositories.daily_index.clone())
            .build()
            .run_to_detected_filings(|item| on_item(&context, item))
            .await;
        let outcome = match detection {
            Ok(detected) => {
                let filing_outcomes = extract_filings(
                    date,
                    &detected,
                    sec_client,
                    repositories,
                    settings,
                    &on_item,
                    &on_error,
                )
                .await;
                let company_outcomes = extract_companies(
                    date,
                    &detected.filings,
                    sec_client,
                    repositories,
                    settings,
                    &on_item,
                    &on_error,
                )
                .await;
                DateOutcome::Processed {
                    filings: filing_outcomes,
                    companies: company_outcomes,
                }
            }
            Err(error) if is_pending(&error, date, first_date, settings.yesterday) => {
                on_error(&context, &error, ErrorKind::Pending);
                DateOutcome::Pending(error)
            }
            Err(error) => {
                on_error(&context, &error, ErrorKind::Failed);
                DateOutcome::Failed(error)
            }
        };
        outcomes.push((date, outcome));
    }
    RunSummary(outcomes)
}

/// Runs one filing machine per detected filing, at most [`IN_FLIGHT`] at once, in the order of
/// the filings. Each machine records the daily index of `detected` as the input of its files.
async fn extract_filings<C>(
    date: NaiveDate,
    detected: &DetectedFilings,
    sec_client: &C,
    repositories: &Repositories,
    settings: &Settings,
    on_item: &(impl Fn(&Value, &StreamItem) + Sync),
    on_error: &(impl Fn(&Value, &StreamError, ErrorKind) + Sync),
) -> Vec<FilingOutcome>
where
    C: DailyClient,
    ExtractFilingSuperState<
        PrepareSecRequest<FilingIndex>,
        InstanceRepository,
        LabelsRepository,
        C,
    >: IntoStateMachineStream,
{
    let inputs: Vec<InputDocument> = detected.index.iter().cloned().collect();
    let inputs = &inputs;
    let extractions = detected.filings.iter().map(|filing| async move {
        let context = json!({
            "date": date.to_string(),
            "accession_number": filing.accession_number().to_string(),
        });
        let result = with_retry(
            &settings.retry_delays,
            || {
                ExtractFilingSuperState::builder()
                    .filing_ref(filing.clone())
                    .sec_client(sec_client.clone())
                    .execution_id(settings.execution_id)
                    .instance_repository(repositories.instance.clone())
                    .labels_repository(repositories.labels.clone())
                    .inputs(inputs.clone())
                    .build()
                    .run_to_documents(|item| on_item(&context, item))
            },
            |error| on_error(&context, error, ErrorKind::Retried),
        )
        .await
        .map(|_| ())
        .inspect_err(|error| {
            let kind = if is_unextractable(error) {
                ErrorKind::Unextractable
            } else {
                ErrorKind::Failed
            };
            on_error(&context, error, kind);
        });
        FilingOutcome {
            accession_number: filing.accession_number().clone(),
            result,
        }
    });
    futures_util::stream::iter(extractions)
        .buffered(IN_FLIGHT)
        .collect()
        .await
}

/// Runs one `submissions` machine per distinct CIK of `filings`, at most [`IN_FLIGHT`] at once,
/// in the order in which each CIK first appears.
async fn extract_companies<C>(
    date: NaiveDate,
    filings: &[FilingRef],
    sec_client: &C,
    repositories: &Repositories,
    settings: &Settings,
    on_item: &(impl Fn(&Value, &StreamItem) + Sync),
    on_error: &(impl Fn(&Value, &StreamError, ErrorKind) + Sync),
) -> Vec<CompanyOutcome>
where
    C: DailyClient,
    ExtractSubmissionsSuperState<ValidateCikFormat, SubmissionsRepository, C>:
        IntoStateMachineStream,
{
    let mut ciks: Vec<&Cik> = Vec::new();
    for filing in filings {
        if !ciks.contains(&filing.cik()) {
            ciks.push(filing.cik());
        }
    }
    let extractions = ciks.into_iter().map(|cik| async move {
        let context = json!({ "date": date.to_string(), "cik": cik.to_string() });
        let result = with_retry(
            &settings.retry_delays,
            || {
                let stream = ExtractSubmissionsSuperState::builder()
                    .cik(cik.to_string())
                    .sec_client(sec_client.clone())
                    .execution_id(settings.execution_id)
                    .ingestion_mode(IngestionMode::Incremental)
                    .repository(repositories.submissions.clone())
                    .build()
                    .into_run_stream();
                run_to_end(stream, |item| on_item(&context, item))
            },
            |error| on_error(&context, error, ErrorKind::Retried),
        )
        .await
        .inspect_err(|error| on_error(&context, error, ErrorKind::Failed));
        CompanyOutcome {
            cik: cik.clone(),
            result,
        }
    });
    futures_util::stream::iter(extractions)
        .buffered(IN_FLIGHT)
        .collect()
        .await
}

/// Runs `stream` to its end, and returns the first error.
async fn run_to_end(
    mut stream: StateMachineStream,
    on_item: impl Fn(&StreamItem),
) -> Result<(), StreamError> {
    while let Some(item) = stream.next().await {
        on_item(&item?);
    }
    Ok(())
}

/// Runs `attempt` until it succeeds, fails with an error that is not a 503, or has used every
/// wait of `retry_delays`. Calls `on_retry` with each error after which `attempt` runs again, and
/// returns the last error.
async fn with_retry<T, F, Fut>(
    retry_delays: &[Duration],
    mut attempt: F,
    on_retry: impl Fn(&StreamError),
) -> Result<T, StreamError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, StreamError>>,
{
    let mut delays = retry_delays.iter();
    loop {
        match attempt().await {
            Ok(value) => return Ok(value),
            Err(error) => match delays.next() {
                Some(delay) if is_unavailable(&error) => {
                    on_retry(&error);
                    tokio::time::sleep(*delay).await;
                }
                _ => return Err(error),
            },
        }
    }
}

/// Returns `true` if the SEC answered 503 to a request of the machine.
pub fn is_unavailable(error: &StreamError) -> bool {
    matches!(
        &error.source,
        StateMachine::State(StateError::FailedRequestExecution(failure))
            if failure.domain_error().is_unavailable()
    )
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use pretty_assertions::assert_eq;
    use sec::error::state_machine::state::{
        FailedDocumentSelection, FailedRequestExecution, MissingDailyIndex,
    };
    use sec::prelude::StreamEvent;
    use sec::shared::filing_documents::FilingDocumentRole;
    use sec::shared::index_date::IndexDate;
    use sec::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use sec::shared::status_code::StatusCode;
    use uuid::Uuid;

    use super::*;

    const ACCESSION_NUMBER: &str = "0000003545-26-000047";

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .expect("Given a hardcoded valid date, parsing should always succeed")
    }

    fn accession_number() -> AccessionNumber {
        AccessionNumber::new(ACCESSION_NUMBER).expect(
            "Given a hardcoded accession number of the right shape, creation should always succeed",
        )
    }

    fn cik() -> Cik {
        Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed")
    }

    fn stream_error(state_name: &str, source: StateMachine) -> StreamError {
        StreamError {
            event: StreamEvent::StateFailed,
            execution_id: Uuid::nil(),
            state_name: state_name.to_string(),
            data: Value::Null,
            source,
        }
    }

    fn missing_daily_index(index_date: &str) -> StreamError {
        let index_date = IndexDate::new(index_date)
            .expect("Given a hardcoded valid date, the index date should always build");
        stream_error(
            "Download Daily Index",
            StateMachine::State(StateError::MissingDailyIndex(MissingDailyIndex::new(
                "Download Daily Index",
                index_date,
            ))),
        )
    }

    fn failed_selection(filing_documents_error: FilingDocumentsError) -> StreamError {
        stream_error(
            "Select Filing Documents",
            StateMachine::State(StateError::FailedDocumentSelection(
                FailedDocumentSelection::new("Select Filing Documents", filing_documents_error),
            )),
        )
    }

    fn missing_instance() -> StreamError {
        failed_selection(FilingDocumentsError::MissingXbrlInstance {
            accession_number: accession_number(),
        })
    }

    fn ambiguous_instance() -> StreamError {
        failed_selection(FilingDocumentsError::AmbiguousFilingDocument {
            accession_number: accession_number(),
            role: FilingDocumentRole::XbrlInstance,
            candidate_count: 2,
        })
    }

    fn request_error(status_code: StatusCode) -> StreamError {
        let failure =
            FailedSecRequest::from(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
                status_code,
            }));
        stream_error(
            "Execute SEC Request",
            StateMachine::State(StateError::FailedRequestExecution(
                FailedRequestExecution::new("Execute SEC Request", failure),
            )),
        )
    }

    /// Returns how many times `with_retry` runs an attempt that always fails with `error`.
    async fn attempts_until_given_up(error: fn() -> StreamError) -> usize {
        let attempts = AtomicUsize::new(0);
        let retry_delays = [Duration::ZERO, Duration::ZERO];
        let _ = with_retry(
            &retry_delays,
            || {
                attempts.fetch_add(1, Ordering::SeqCst);
                async move { Err::<(), _>(error()) }
            },
            |_| {},
        )
        .await;
        attempts.load(Ordering::SeqCst)
    }

    fn unavailable() -> StreamError {
        request_error(StatusCode::ServiceUnavailable)
    }

    fn not_found() -> StreamError {
        request_error(StatusCode::NotFound)
    }

    #[test]
    fn should_write_each_kind_in_lowercase_when_displayed() {
        let kinds = [
            ErrorKind::Retried,
            ErrorKind::Pending,
            ErrorKind::Unextractable,
            ErrorKind::Failed,
        ];

        let expected_result = vec!["retried", "pending", "unextractable", "failed"];

        let result: Vec<String> = kinds.iter().map(ToString::to_string).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_an_error_as_unavailable_when_the_sec_answers_503() {
        let error = unavailable();

        let expected_result = true;

        let result = is_unavailable(&error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_count_an_error_as_unavailable_when_the_sec_answers_404() {
        let error = not_found();

        let expected_result = false;

        let result = is_unavailable(&error);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_try_once_more_per_retry_delay_when_every_attempt_gets_503() {
        let error = unavailable;

        let expected_result = 3;

        let result = attempts_until_given_up(error).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_try_once_when_the_error_is_not_a_503() {
        let error = not_found;

        let expected_result = 1;

        let result = attempts_until_given_up(error).await;

        assert_eq!(result, expected_result);
    }

    fn configuration_error() -> StreamError {
        stream_error("Validate CIK Format", StateMachine::InvalidConfiguration)
    }

    fn processed(
        filing_results: Vec<Result<(), StreamError>>,
        company_results: Vec<Result<(), StreamError>>,
    ) -> DateOutcome {
        DateOutcome::Processed {
            filings: filing_results
                .into_iter()
                .map(|result| FilingOutcome {
                    accession_number: accession_number(),
                    result,
                })
                .collect(),
            companies: company_results
                .into_iter()
                .map(|result| CompanyOutcome { cik: cik(), result })
                .collect(),
        }
    }

    #[test]
    fn should_count_a_missing_index_as_pending_when_the_date_is_two_days_before_yesterday() {
        let run_date = date("2026-10-29");
        let error = missing_daily_index("2026-10-29");
        let first_date = date("2026-10-28");
        let yesterday = date("2026-10-31");

        let expected_result = true;

        let result = is_pending(&error, run_date, first_date, yesterday);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_count_a_missing_index_as_pending_when_the_date_is_three_days_before_yesterday() {
        let run_date = date("2026-10-28");
        let error = missing_daily_index("2026-10-28");
        let first_date = date("2026-10-27");
        let yesterday = date("2026-10-31");

        let expected_result = false;

        let result = is_pending(&error, run_date, first_date, yesterday);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_count_a_missing_index_as_pending_when_the_date_is_the_first_date() {
        let run_date = date("2026-10-31");
        let error = missing_daily_index("2026-10-31");
        let first_date = date("2026-10-31");
        let yesterday = date("2026-10-31");

        let expected_result = false;

        let result = is_pending(&error, run_date, first_date, yesterday);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_count_another_detection_error_as_pending_when_the_date_is_yesterday() {
        let run_date = date("2026-10-31");
        let error = configuration_error();
        let first_date = date("2026-10-28");
        let yesterday = date("2026-10-31");

        let expected_result = false;

        let result = is_pending(&error, run_date, first_date, yesterday);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_a_filing_as_unextractable_when_its_folder_holds_no_instance() {
        let error = missing_instance();

        let expected_result = true;

        let result = is_unextractable(&error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_count_a_filing_as_unextractable_when_its_folder_holds_two_instances() {
        let error = ambiguous_instance();

        let expected_result = false;

        let result = is_unextractable(&error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_count_a_filing_as_unextractable_when_another_error_failed_it() {
        let error = configuration_error();

        let expected_result = false;

        let result = is_unextractable(&error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_each_failed_filing_and_submissions_file_when_a_date_is_processed() {
        let summary = RunSummary(vec![(
            date("2026-10-30"),
            processed(
                vec![Err(ambiguous_instance())],
                vec![Err(configuration_error())],
            ),
        )]);

        let expected_result = vec![json!({
            "date": "2026-10-30",
            "detection": "processed",
            "filings": {
                "selected": 1,
                "stored": 0,
                "unextractable": [],
                "failed": [{
                    "accession_number": "0000003545-26-000047",
                    "failed_state": "Select Filing Documents",
                    "error": "[StreamError][00000000-0000-0000-0000-000000000000] 'state_failed' in 'Select Filing Documents', Caused by: Problem occurred during internal state operations: '[StateError] A state level error occurred, Caused by: [FailedDocumentSelection] Failure in State: 'Select Filing Documents', Caused by: [AmbiguousFilingDocument] The folder of filing '0000003545-26-000047' holds 2 candidates for the XBRL instance'",
                }],
            },
            "submissions": {
                "companies": 1,
                "stored": 0,
                "failed": [{
                    "cik": "0000003545",
                    "failed_state": "Validate CIK Format",
                    "error": "[StreamError][00000000-0000-0000-0000-000000000000] 'state_failed' in 'Validate CIK Format', Caused by: Invalid configuration of the state machine",
                }],
            },
        })];

        let result = summary.entries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_ok_when_only_pending_dates_and_unextractable_filings_did_not_store() {
        let summary = RunSummary(vec![
            (
                date("2026-10-30"),
                processed(vec![Ok(()), Err(missing_instance())], vec![Ok(())]),
            ),
            (
                date("2026-10-31"),
                DateOutcome::Pending(missing_daily_index("2026-10-31")),
            ),
        ]);

        let expected_result = Ok(());

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_each_failed_date_filing_and_submissions_file_when_the_run_has_failures() {
        let summary = RunSummary(vec![
            (
                date("2026-10-29"),
                DateOutcome::Failed(configuration_error()),
            ),
            (
                date("2026-10-30"),
                processed(
                    vec![Err(ambiguous_instance())],
                    vec![Err(configuration_error())],
                ),
            ),
        ]);

        let expected_result = Err(FailedDailyRun {
            failures: 3,
            dates: 2,
        });

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_failures_and_the_dates_when_displaying_a_failed_run() {
        let error = FailedDailyRun {
            failures: 3,
            dates: 2,
        };

        let expected_result =
            "[FailedDailyRun] A run of 2 dates has 3 failed dates, filings, or submissions files";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_empty_when_detection_selected_no_filing() {
        let summary = RunSummary(vec![(date("2026-10-31"), processed(vec![], vec![]))]);

        let expected_result = vec![json!({
            "date": "2026-10-31",
            "detection": "empty",
            "filings": { "selected": 0, "stored": 0, "unextractable": [], "failed": [] },
            "submissions": { "companies": 0, "stored": 0, "failed": [] },
        })];

        let result = summary.entries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_an_unextractable_filing_apart_from_the_failed_ones_when_a_date_is_processed() {
        let summary = RunSummary(vec![(
            date("2026-10-30"),
            processed(vec![Ok(()), Err(missing_instance())], vec![Ok(())]),
        )]);

        let expected_result = vec![json!({
            "date": "2026-10-30",
            "detection": "processed",
            "filings": {
                "selected": 2,
                "stored": 1,
                "unextractable": [{
                    "accession_number": "0000003545-26-000047",
                    "failed_state": "Select Filing Documents",
                    "error": "[StreamError][00000000-0000-0000-0000-000000000000] 'state_failed' in 'Select Filing Documents', Caused by: Problem occurred during internal state operations: '[StateError] A state level error occurred, Caused by: [FailedDocumentSelection] Failure in State: 'Select Filing Documents', Caused by: [MissingXbrlInstance] The folder of filing '0000003545-26-000047' holds no XBRL instance'",
                }],
                "failed": [],
            },
            "submissions": { "companies": 1, "stored": 1, "failed": [] },
        })];

        let result = summary.entries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_pending_with_its_error_when_the_index_does_not_exist_yet() {
        let summary = RunSummary(vec![(
            date("2026-10-31"),
            DateOutcome::Pending(missing_daily_index("2026-10-31")),
        )]);

        let expected_result = vec![json!({
            "date": "2026-10-31",
            "detection": "pending",
            "error": "[StreamError][00000000-0000-0000-0000-000000000000] 'state_failed' in 'Download Daily Index', Caused by: Problem occurred during internal state operations: '[StateError] A state level error occurred, Caused by: [MissingDailyIndex] Failure in State: 'Download Daily Index', Reason: 'The SEC sent no daily index of 2026-10-31, and no quarter listing shows that the day had no filings''",
        })];

        let result = summary.entries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_failed_state_when_detection_failed() {
        let summary = RunSummary(vec![(
            date("2026-10-29"),
            DateOutcome::Failed(configuration_error()),
        )]);

        let expected_result = vec![json!({
            "date": "2026-10-29",
            "detection": "failed",
            "failed_state": "Validate CIK Format",
            "error": "[StreamError][00000000-0000-0000-0000-000000000000] 'state_failed' in 'Validate CIK Format', Caused by: Invalid configuration of the state machine",
        })];

        let result = summary.entries();

        assert_eq!(result, expected_result);
    }
}
