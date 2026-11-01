//! # Daily Filings
//!
//! Provides [`IncrementalDailyFilings`], the new-filing path of an incremental run over many
//! dates, and the [`DailyRunSummary`] of that run.
//!
//! [`IncrementalDailyFilings::dates`] returns a [`DailyFilingsBuilder`], whose [`run`] runs the
//! dates in order, one date at a time. For each date, the run detects the filings, then extracts
//! each filing, then the `submissions` file of each company. Before it sends a request for a
//! filing, the run reads the stored XBRL instance of the filing. If the instance passes the size
//! and SHA-256 check, the run skips the filing.
//!
//! [`run`]: DailyFilingsBuilder::run

use std::future::Future;
use std::time::Duration;

use chrono::NaiveDate;
use futures_util::StreamExt;
use storage::implementations::repository::filesystem::{
    FilesystemRepository, InputDocument, Read, Write,
};
use storage::traits::repository::ReadRepository;

use super::Incremental;
use super::run_event::{ErrorKind, RunEvent, Unit};
use crate::implementations::states::extract::daily_index::builder::NoSecClient;
use crate::implementations::states::extract::daily_index::{
    DetectFilingsSuperState, DetectedFilings,
};
use crate::implementations::states::extract::filing::ExtractFilingSuperState;
use crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
use crate::implementations::states::extract::submissions::ExtractSubmissionsSuperState;
use crate::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use crate::implementations::states::extract::validate_index_date::ValidateIndexDate;
use crate::prelude::{IntoStateMachineStream, StateMachineStream, StreamError, StreamItem};
use crate::shared::cik::Cik;
use crate::shared::filing_documents_ref::instance_key;
use crate::shared::filing_ref::FilingRef;
use crate::shared::http_client::Executes;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::{RawDailyIndex, RawSubmissions, RawXbrlInstance, RawXbrlLabels};
use crate::shared::request::implementations::sec_request::{FilingDocumentSet, FilingIndex};
use crate::shared::response::implementations::sec_response::{JsonBody, SecResponse, ZipBody};
use summary::{
    CompanyOutcome, DateOutcome, Fetch, FilingOutcome, is_pending, is_unavailable, is_unextractable,
};

mod summary;

pub use summary::{DailyRunSummary, FailedDailyRun};

/// The number of filing machines, and of `submissions` machines, that one date runs at once
/// without a call to [`DailyFilingsBuilder::in_flight`].
///
/// The rate limiter of the SEC client caps the requests, so a higher value does not make a run
/// faster.
pub const DEFAULT_MACHINES_IN_FLIGHT: usize = 4;

/// The waits before the second and the third attempt of a machine that the SEC answered with 503,
/// without a call to [`DailyFilingsBuilder::retry_delays`]. After the third attempt the machine
/// fails.
// Deviation: `Duration::from_mins` is not stable in Rust 1.90, the CI version, so the code writes
// the minute in seconds. Rust 1.90 does not know the lint, hence `unknown_lints`.
#[allow(unknown_lints, clippy::duration_suboptimal_units)]
pub const DEFAULT_RETRY_DELAYS: [Duration; 2] = [Duration::from_secs(10), Duration::from_secs(60)];

/// The number of dates, yesterday and the days before it, whose missing daily index counts as
/// pending. A pending date does not fail the run, because a later run reads it again.
pub const PENDING_DAYS: u64 = 3;

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
type InstanceReader = FilesystemRepository<Read, RawDocument<RawXbrlInstance>>;
type LabelsRepository = FilesystemRepository<Write, RawDocument<RawXbrlLabels>>;
type SubmissionsRepository = FilesystemRepository<Write, RawDocument<RawSubmissions>>;

/// The stores that one run writes to.
#[derive(Debug, Clone)]
pub struct DailyFilingsRepositories {
    /// The store of the daily indexes.
    pub daily_index: DailyIndexRepository,
    /// The store of the XBRL instances.
    pub instance: InstanceRepository,
    /// The store of the label files.
    pub labels: LabelsRepository,
    /// The store of the `submissions` files.
    pub submissions: SubmissionsRepository,
}

/// Marks a [`DailyFilingsBuilder`] that holds no `yesterday` yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoYesterday;

/// Marks a [`DailyFilingsBuilder`] that holds no stores yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoRepositories;

/// The new-filing path of an incremental run, before the choice of the dates.
///
/// Every super-state of the run carries the `execution_id` of its [`Incremental`] handle and
/// [`IngestionMode::Incremental`](storage::implementations::repository::filesystem::IngestionMode::Incremental).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IncrementalDailyFilings {
    incremental: Incremental,
}

impl IncrementalDailyFilings {
    /// Creates a new [`IncrementalDailyFilings`] for the run of `incremental`.
    pub(super) const fn new(incremental: Incremental) -> Self {
        Self { incremental }
    }

    /// Returns the builder of a run over `dates`. The run reads them oldest first, once each.
    ///
    /// The pending rule never applies to the oldest date. The builder still needs yesterday, an
    /// SEC client, and the stores.
    #[must_use]
    pub fn dates(
        self,
        dates: impl IntoIterator<Item = NaiveDate>,
    ) -> DailyFilingsBuilder<NoYesterday, NoSecClient, NoRepositories> {
        let mut dates: Vec<NaiveDate> = dates.into_iter().collect();
        dates.sort_unstable();
        dates.dedup();
        DailyFilingsBuilder {
            incremental: self.incremental,
            dates,
            yesterday: NoYesterday,
            sec_client: NoSecClient,
            repositories: NoRepositories,
            in_flight: DEFAULT_MACHINES_IN_FLIGHT,
            retry_delays: DEFAULT_RETRY_DELAYS.to_vec(),
        }
    }
}

/// Builds and runs the new-filing path over many dates.
///
/// Set [`yesterday`](Self::yesterday), [`sec_client`](Self::sec_client), and
/// [`repositories`](Self::repositories), in any order. [`in_flight`](Self::in_flight) and
/// [`retry_delays`](Self::retry_delays) are optional. Then call [`run`](Self::run).
///
/// `run` needs the stores:
///
/// ```compile_fail,E0599
/// use chrono::NaiveDate;
/// use sec::implementations::extraction::Extraction;
/// use sec::shared::http_client::implementations::sec_client::SecClient;
///
/// let yesterday = NaiveDate::MIN;
/// let _run = Extraction::incremental()
///     .daily_filings()
///     .dates([yesterday])
///     .yesterday(yesterday)
///     .sec_client(SecClient::default())
///     .run(|_, _| {});
/// ```
///
/// `run` needs the SEC client:
///
/// ```compile_fail,E0599
/// use chrono::NaiveDate;
/// use sec::implementations::extraction::{DailyFilingsRepositories, Extraction};
///
/// fn run_without_sec_client(repositories: DailyFilingsRepositories) {
///     let _run = Extraction::incremental()
///         .daily_filings()
///         .dates([NaiveDate::MIN])
///         .yesterday(NaiveDate::MIN)
///         .repositories(repositories)
///         .run(|_, _| {});
/// }
/// ```
///
/// And `run` needs yesterday:
///
/// ```compile_fail,E0599
/// use chrono::NaiveDate;
/// use sec::implementations::extraction::{DailyFilingsRepositories, Extraction};
/// use sec::shared::http_client::implementations::sec_client::SecClient;
///
/// fn run_without_yesterday(repositories: DailyFilingsRepositories) {
///     let _run = Extraction::incremental()
///         .daily_filings()
///         .dates([NaiveDate::MIN])
///         .sec_client(SecClient::default())
///         .repositories(repositories)
///         .run(|_, _| {});
/// }
/// ```
#[derive(Debug, Clone)]
pub struct DailyFilingsBuilder<Y, C, R> {
    incremental: Incremental,
    dates: Vec<NaiveDate>,
    yesterday: Y,
    sec_client: C,
    repositories: R,
    in_flight: usize,
    retry_delays: Vec<Duration>,
}

impl<C, R> DailyFilingsBuilder<NoYesterday, C, R> {
    /// Sets yesterday in New York, the last date that the pending rule covers.
    #[must_use]
    pub fn yesterday(self, yesterday: NaiveDate) -> DailyFilingsBuilder<NaiveDate, C, R> {
        DailyFilingsBuilder {
            incremental: self.incremental,
            dates: self.dates,
            yesterday,
            sec_client: self.sec_client,
            repositories: self.repositories,
            in_flight: self.in_flight,
            retry_delays: self.retry_delays,
        }
    }
}

impl<Y, R> DailyFilingsBuilder<Y, NoSecClient, R> {
    /// Sets the SEC client of every machine of the run.
    #[must_use]
    pub fn sec_client<C: DailyClient>(self, sec_client: C) -> DailyFilingsBuilder<Y, C, R> {
        DailyFilingsBuilder {
            incremental: self.incremental,
            dates: self.dates,
            yesterday: self.yesterday,
            sec_client,
            repositories: self.repositories,
            in_flight: self.in_flight,
            retry_delays: self.retry_delays,
        }
    }
}

impl<Y, C> DailyFilingsBuilder<Y, C, NoRepositories> {
    /// Sets the stores of the daily indexes, the filing documents, and the `submissions` files.
    #[must_use]
    pub fn repositories(
        self,
        repositories: DailyFilingsRepositories,
    ) -> DailyFilingsBuilder<Y, C, DailyFilingsRepositories> {
        DailyFilingsBuilder {
            incremental: self.incremental,
            dates: self.dates,
            yesterday: self.yesterday,
            sec_client: self.sec_client,
            repositories,
            in_flight: self.in_flight,
            retry_delays: self.retry_delays,
        }
    }
}

impl<Y, C, R> DailyFilingsBuilder<Y, C, R> {
    /// Sets the most filing machines, and the most `submissions` machines, that one date runs at
    /// once. The default is [`DEFAULT_MACHINES_IN_FLIGHT`], and `0` counts as `1`.
    #[must_use]
    pub fn in_flight(self, in_flight: usize) -> Self {
        Self {
            in_flight: in_flight.max(1),
            ..self
        }
    }

    /// Sets the waits before each new attempt of a machine that the SEC answered with 503. The
    /// default is [`DEFAULT_RETRY_DELAYS`].
    #[must_use]
    pub fn retry_delays(self, retry_delays: impl IntoIterator<Item = Duration>) -> Self {
        Self {
            retry_delays: retry_delays.into_iter().collect(),
            ..self
        }
    }
}

impl<C> DailyFilingsBuilder<NaiveDate, C, DailyFilingsRepositories>
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
    /// Runs the new-filing path for each date, in order, and returns the summary.
    ///
    /// The dates run one after another, never at once. For each date, the run runs detection,
    /// then one filing machine per selected filing that the store does not hold, then one
    /// `submissions` machine per distinct CIK of the selected filings. At most
    /// [`in_flight`](Self::in_flight) filing machines run at once, and the same for `submissions`
    /// machines. A filing or `submissions` machine that the SEC answers with 503 runs again after
    /// each retry delay. A failure does not stop the other machines or the other dates.
    ///
    /// Calls `observer` with the unit and each successful stream item, and with the unit, each
    /// error, and its [`ErrorKind`]. The unit is the date, the filing, or the company. A filing
    /// that the store holds sends no event to the observer.
    pub async fn run(
        self,
        observer: impl Fn(&Unit, RunEvent<'_>) + Send + Sync + 'static,
    ) -> DailyRunSummary {
        let Some(&first_date) = self.dates.first() else {
            return DailyRunSummary(Vec::new());
        };
        let mut outcomes = Vec::with_capacity(self.dates.len());
        for &date in &self.dates {
            let unit = Unit::Date(date);
            let detection = self
                .incremental
                .daily_index()
                .date(date.to_string())
                .sec_client(self.sec_client.clone())
                .repository(self.repositories.daily_index.clone())
                .build()
                .run_to_detected_filings(|item| observer(&unit, RunEvent::Item(item)))
                .await;
            let outcome = match detection {
                Ok(detected) => DateOutcome::Processed {
                    filings: self.extract_filings(date, &detected, &observer).await,
                    companies: self
                        .extract_companies(date, &detected.filings, &observer)
                        .await,
                },
                Err(error) if is_pending(&error, date, first_date, self.yesterday) => {
                    observer(&unit, RunEvent::Error(&error, ErrorKind::Pending));
                    DateOutcome::Pending(error)
                }
                Err(error) => {
                    observer(&unit, RunEvent::Error(&error, ErrorKind::Failed));
                    DateOutcome::Failed(error)
                }
            };
            outcomes.push((date, outcome));
        }
        DailyRunSummary(outcomes)
    }

    /// Runs one filing machine per detected filing, at most the in-flight cap at once, in the
    /// order of the filings. Each machine records the daily index of `detected` as the input of
    /// its files.
    ///
    /// A filing whose stored instance passes the size and SHA-256 check runs no machine. A
    /// stored instance that fails the check, or that the store cannot read, runs the machine
    /// again.
    async fn extract_filings(
        &self,
        date: NaiveDate,
        detected: &DetectedFilings,
        observer: &(impl Fn(&Unit, RunEvent<'_>) + Sync),
    ) -> Vec<FilingOutcome> {
        let inputs: Vec<InputDocument> = detected.index.iter().cloned().collect();
        let inputs = &inputs;
        let reader = &FilesystemRepository::builder()
            .reader()
            .document_type::<RawDocument<RawXbrlInstance>>()
            .root(self.repositories.instance.root())
            .build();
        let extractions = detected.filings.iter().map(|filing| async move {
            if holds_instance(reader, filing).await {
                return FilingOutcome {
                    accession_number: filing.accession_number().clone(),
                    result: Ok(Fetch::AlreadyStored),
                };
            }
            let unit = Unit::Filing {
                date,
                accession_number: filing.accession_number().clone(),
            };
            let result = with_retry(
                &self.retry_delays,
                || {
                    self.incremental
                        .filing()
                        .filing_ref(filing.clone())
                        .sec_client(self.sec_client.clone())
                        .instance_repository(self.repositories.instance.clone())
                        .labels_repository(self.repositories.labels.clone())
                        .inputs(inputs.clone())
                        .build()
                        .run_to_documents(|item| observer(&unit, RunEvent::Item(item)))
                },
                |error| observer(&unit, RunEvent::Error(error, ErrorKind::Retried)),
            )
            .await
            .map(|_| Fetch::Downloaded)
            .inspect_err(|error| {
                let kind = if is_unextractable(error) {
                    ErrorKind::Unextractable
                } else {
                    ErrorKind::Failed
                };
                observer(&unit, RunEvent::Error(error, kind));
            });
            FilingOutcome {
                accession_number: filing.accession_number().clone(),
                result,
            }
        });
        futures_util::stream::iter(extractions)
            .buffered(self.in_flight)
            .collect()
            .await
    }

    /// Runs one `submissions` machine per distinct CIK of `filings`, at most the in-flight cap at
    /// once, in the order in which each CIK first appears.
    async fn extract_companies(
        &self,
        date: NaiveDate,
        filings: &[FilingRef],
        observer: &(impl Fn(&Unit, RunEvent<'_>) + Sync),
    ) -> Vec<CompanyOutcome> {
        let mut ciks: Vec<&Cik> = Vec::new();
        for filing in filings {
            if !ciks.contains(&filing.cik()) {
                ciks.push(filing.cik());
            }
        }
        let extractions = ciks.into_iter().map(|cik| async move {
            let unit = Unit::Company {
                date,
                cik: cik.clone(),
            };
            let result = with_retry(
                &self.retry_delays,
                || {
                    let stream = self
                        .incremental
                        .submissions()
                        .cik(cik.to_string())
                        .sec_client(self.sec_client.clone())
                        .repository(self.repositories.submissions.clone())
                        .build()
                        .into_run_stream();
                    run_to_end(stream, |item| observer(&unit, RunEvent::Item(item)))
                },
                |error| observer(&unit, RunEvent::Error(error, ErrorKind::Retried)),
            )
            .await
            .inspect_err(|error| observer(&unit, RunEvent::Error(error, ErrorKind::Failed)));
            CompanyOutcome {
                cik: cik.clone(),
                result,
            }
        });
        futures_util::stream::iter(extractions)
            .buffered(self.in_flight)
            .collect()
            .await
    }
}

/// Returns `true` if `reader` holds the XBRL instance of `filing`, and the instance passes the
/// size and SHA-256 check.
///
/// A read error returns `false`, so the run fetches the filing again.
async fn holds_instance(reader: &InstanceReader, filing: &FilingRef) -> bool {
    matches!(reader.get(instance_key(filing)).await, Ok(Some(_)))
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

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use pretty_assertions::assert_eq;

    use super::summary::tests::{not_found, unavailable};
    use super::*;
    use crate::implementations::extraction::Extraction;

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

    #[test]
    fn should_count_zero_as_one_when_setting_in_flight() {
        let in_flight = 0;

        let expected_result = 1;

        let result = Extraction::incremental()
            .daily_filings()
            .dates([NaiveDate::MIN])
            .in_flight(in_flight)
            .in_flight;

        assert_eq!(result, expected_result);
    }
}
