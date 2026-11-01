//! # Summary
//!
//! Provides [`DailyRunSummary`], the outcome of every date of one `daily_filings` run, and the rules
//! that sort an error into an outcome.
//!
//! A date is pending if its daily index does not exist yet and a later run reads the date again.
//! A filing is unextractable if its folder holds no XBRL instance. Neither fails the run.

use chrono::{Days, NaiveDate};
use serde_json::{Value, json};
use thiserror::Error;

use super::PENDING_DAYS;
use crate::error::{State as StateError, StateMachine};
use crate::prelude::StreamError;
use crate::shared::cik::Cik;
use crate::shared::filing_documents::FilingDocumentsError;
use crate::shared::financial::accession_number::AccessionNumber;

/// The outcome of one date.
#[derive(Debug)]
pub(super) enum DateOutcome {
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

/// How the run got the stored documents of a filing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Fetch {
    /// The run downloaded and stored the documents.
    Downloaded,
    /// The store already held an intact instance, so the run sent no request for the filing.
    AlreadyStored,
}

/// The outcome of one filing: how the run got its documents, or the error of the state that
/// failed.
#[derive(Debug)]
pub(super) struct FilingOutcome {
    /// The accession number of the filing.
    pub(super) accession_number: AccessionNumber,
    /// `Ok` if the store holds the documents of the filing after the run.
    pub(super) result: Result<Fetch, StreamError>,
}

/// The outcome of the `submissions` file of one company.
#[derive(Debug)]
pub(super) struct CompanyOutcome {
    /// The CIK of the company.
    pub(super) cik: Cik,
    /// `Ok` if the run stored the `submissions` file.
    pub(super) result: Result<(), StreamError>,
}

/// The outcome of every date of one run, oldest date first.
#[derive(Debug)]
pub struct DailyRunSummary(pub(super) Vec<(NaiveDate, DateOutcome)>);

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
pub(super) fn is_pending(
    error: &StreamError,
    date: NaiveDate,
    first_date: NaiveDate,
    yesterday: NaiveDate,
) -> bool {
    // A `yesterday` near `NaiveDate::MIN` has no oldest pending date, so every date is recent.
    let is_recent = yesterday
        .checked_sub_days(Days::new(PENDING_DAYS - 1))
        .is_none_or(|oldest_pending| date >= oldest_pending);
    date != first_date
        && is_recent
        && matches!(
            &error.source,
            StateMachine::State(StateError::MissingDailyIndex(_))
        )
}

/// Returns `true` if the folder of the filing holds no XBRL instance.
///
/// A filing like that fails the same way on every attempt, so it does not fail the run. A folder
/// with more than one candidate fails the run, because it shows a gap in the selection rules.
pub(super) const fn is_unextractable(error: &StreamError) -> bool {
    matches!(
        &error.source,
        StateMachine::State(StateError::FailedDocumentSelection(failure))
            if matches!(
                failure.filing_documents_error,
                FilingDocumentsError::MissingXbrlInstance { .. }
            )
    )
}

impl DailyRunSummary {
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
    /// selected), `pending`, or `failed`. A processed date lists the counts of selected, stored,
    /// and already stored filings, the counts of `submissions` files, and each unextractable or
    /// failed one with its error. A stored filing is one the run downloaded. An already stored
    /// filing is one the store held before the run, so the run sent no request for it.
    #[must_use]
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
    let fetched_by = |fetch: Fetch| {
        filings
            .iter()
            .filter(|filing| filing.result.as_ref().is_ok_and(|found| *found == fetch))
            .count()
    };
    json!({
        "selected": filings.len(),
        "stored": fetched_by(Fetch::Downloaded),
        "already_stored": fetched_by(Fetch::AlreadyStored),
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

/// Returns `true` if the SEC answered 503 to a request of the machine.
pub(super) fn is_unavailable(error: &StreamError) -> bool {
    matches!(
        &error.source,
        StateMachine::State(StateError::FailedRequestExecution(failure))
            if failure.domain_error().is_unavailable()
    )
}

#[cfg(test)]
pub(super) mod tests {
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::error::state_machine::state::{
        FailedDocumentSelection, FailedRequestExecution, MissingDailyIndex,
    };
    use crate::prelude::StreamEvent;
    use crate::shared::filing_documents::FilingDocumentRole;
    use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
    use crate::shared::index_date::IndexDate;
    use crate::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use crate::shared::status_code::StatusCode;

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

    pub(in super::super) fn unavailable() -> StreamError {
        request_error(StatusCode::ServiceUnavailable)
    }

    pub(in super::super) fn not_found() -> StreamError {
        request_error(StatusCode::NotFound)
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

    fn configuration_error() -> StreamError {
        stream_error("Validate CIK Format", StateMachine::InvalidConfiguration)
    }

    fn processed(
        filing_results: Vec<Result<Fetch, StreamError>>,
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
    fn should_count_a_missing_index_as_pending_when_yesterday_is_the_minimum_date() {
        let run_date = NaiveDate::MIN;
        let error = missing_daily_index("2026-10-31");
        let first_date = date("2026-10-28");
        let yesterday = NaiveDate::MIN;

        let expected_result = true;

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
        let summary = DailyRunSummary(vec![(
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
                "already_stored": 0,
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
        let summary = DailyRunSummary(vec![
            (
                date("2026-10-30"),
                processed(
                    vec![Ok(Fetch::Downloaded), Err(missing_instance())],
                    vec![Ok(())],
                ),
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
        let summary = DailyRunSummary(vec![
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
        let summary = DailyRunSummary(vec![(date("2026-10-31"), processed(vec![], vec![]))]);

        let expected_result = vec![json!({
            "date": "2026-10-31",
            "detection": "empty",
            "filings": {
                "selected": 0,
                "stored": 0,
                "already_stored": 0,
                "unextractable": [],
                "failed": [],
            },
            "submissions": { "companies": 0, "stored": 0, "failed": [] },
        })];

        let result = summary.entries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_an_unextractable_filing_apart_from_the_failed_ones_when_a_date_is_processed() {
        let summary = DailyRunSummary(vec![(
            date("2026-10-30"),
            processed(
                vec![Ok(Fetch::Downloaded), Err(missing_instance())],
                vec![Ok(())],
            ),
        )]);

        let expected_result = vec![json!({
            "date": "2026-10-30",
            "detection": "processed",
            "filings": {
                "selected": 2,
                "stored": 1,
                "already_stored": 0,
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
    fn should_count_an_already_stored_filing_apart_from_a_downloaded_one_when_a_date_is_processed()
    {
        let summary = DailyRunSummary(vec![(
            date("2026-10-30"),
            processed(
                vec![Ok(Fetch::AlreadyStored), Ok(Fetch::Downloaded)],
                vec![Ok(())],
            ),
        )]);

        let expected_result = json!({
            "selected": 2,
            "stored": 1,
            "already_stored": 1,
            "unextractable": [],
            "failed": [],
        });

        let result = summary.entries()[0]["filings"].clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_pending_with_its_error_when_the_index_does_not_exist_yet() {
        let summary = DailyRunSummary(vec![(
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
        let summary = DailyRunSummary(vec![(
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
