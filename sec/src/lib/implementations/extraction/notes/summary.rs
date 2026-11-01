//! # Summary
//!
//! Provides [`NotesRunSummary`], the outcome of every period of one Notes batch run.

use chrono::NaiveDate;
use serde_json::{Value, json};
use thiserror::Error;

use super::MAX_UNPUBLISHED;
use crate::error::{State as StateError, StateMachine};
use crate::prelude::StreamError;
use crate::shared::notes_archive_ref::NotesArchiveRef;

/// How the run got the stored file of a period.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fetch {
    /// The run downloaded and stored the file.
    Downloaded,
    /// The store already held an intact file, so the run skipped the download.
    AlreadyStored,
    /// The store held a file that it could not read or that failed the size or SHA-256 check, so
    /// the run downloaded the file again.
    Replaced,
}

/// The outcome of one period: the stored archive and how the run got it, or the error of the state
/// that failed.
pub type PeriodOutcome = Result<(Fetch, NotesArchiveRef), StreamError>;

/// Returns `true` if `error` comes from an SEC answer of 403 or 404 for the Notes file.
pub fn is_absent_file(error: &StreamError) -> bool {
    matches!(
        &error.source,
        StateMachine::State(StateError::FailedRequestExecution(failure))
            if failure.domain_error().is_absent_file()
    )
}

/// Error indicating a Notes run in which at least one period failed.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("[FailedBackfill] {failures} of {periods} Notes periods failed")]
pub struct FailedBackfill {
    /// The number of periods that failed.
    pub failures: usize,
    /// The number of periods in the run.
    pub periods: usize,
}

/// Error indicating that a Notes run gives no cut-over date.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MissingCutOverDate {
    /// At least one period failed, so the stored periods can have a gap.
    #[error("[MissingCutOverDate] The Notes run has no cut-over date, Caused by: {0}")]
    FailedBackfill(#[from] FailedBackfill),
    /// No period of the run has a stored file.
    #[error(
        "[MissingCutOverDate] The Notes run has no cut-over date, Reason: 'no Notes period is stored'"
    )]
    MissingStoredPeriod,
}

/// The outcome of every period of one Notes batch run, in the order of the given periods.
///
/// The SEC posts the file of a month some days into the next month. So the last period of a run
/// can have no file yet. If its file is absent, the summary counts it as unpublished, not as
/// failed. The next run asks for it again.
#[derive(Debug)]
pub struct NotesRunSummary(pub(super) Vec<(String, PeriodOutcome)>);

impl NotesRunSummary {
    /// Returns `Ok` if every period stored its file or is unpublished.
    ///
    /// # Errors
    ///
    /// Returns [`FailedBackfill`] if at least one period failed.
    pub fn result(&self) -> Result<(), FailedBackfill> {
        match self.failure_count() {
            0 => Ok(()),
            failures => Err(FailedBackfill {
                failures,
                periods: self.0.len(),
            }),
        }
    }

    /// Returns the last day of the newest stored period, the date where the new-filing path starts.
    ///
    /// # Errors
    ///
    /// Returns [`MissingCutOverDate::FailedBackfill`] if at least one period failed, because the
    /// stored periods can then have a gap. Returns [`MissingCutOverDate::MissingStoredPeriod`] if
    /// no period has a stored file.
    pub fn cut_over_date(&self) -> Result<NaiveDate, MissingCutOverDate> {
        self.result()?;
        self.0
            .iter()
            .filter_map(|(_, outcome)| outcome.as_ref().ok())
            .map(|(_, archive)| archive.period())
            .max()
            .map(|period| period.last_day())
            .ok_or(MissingCutOverDate::MissingStoredPeriod)
    }

    /// Returns the number of periods that have a stored file, whether downloaded, already stored, or replaced.
    #[must_use]
    pub fn success_count(&self) -> usize {
        self.0.iter().filter(|(_, outcome)| outcome.is_ok()).count()
    }

    /// Returns the number of periods whose intact file the store already held.
    #[must_use]
    pub fn already_stored_count(&self) -> usize {
        self.0
            .iter()
            .filter(|(_, outcome)| matches!(outcome, Ok((Fetch::AlreadyStored, _))))
            .count()
    }

    /// Returns the number of periods that failed and are not unpublished.
    #[must_use]
    pub fn failure_count(&self) -> usize {
        self.0.len() - self.success_count() - self.unpublished_count()
    }

    /// Returns the number of periods at the end of the run whose file is absent, at most
    /// [`MAX_UNPUBLISHED`].
    #[must_use]
    pub fn unpublished_count(&self) -> usize {
        self.0
            .iter()
            .rev()
            .take(MAX_UNPUBLISHED)
            .take_while(|(_, outcome)| outcome.as_ref().is_err_and(is_absent_file))
            .count()
    }

    /// Returns one JSON entry per period.
    ///
    /// A stored period lists its status (`downloaded`, `already_stored`, or `replaced`), key, size,
    /// and SHA-256.
    /// An unpublished period says so. A failed period lists the failed state and the error.
    #[must_use]
    pub fn entries(&self) -> Vec<Value> {
        let first_unpublished = self.0.len() - self.unpublished_count();
        self.0
            .iter()
            .enumerate()
            .map(|(index, (period, outcome))| match outcome {
                Err(_) if index >= first_unpublished => json!({
                    "period": period,
                    "status": "unpublished",
                }),
                Ok((fetch, archive)) => json!({
                    "period": period,
                    "status": match fetch {
                        Fetch::Downloaded => "downloaded",
                        Fetch::AlreadyStored => "already_stored",
                        Fetch::Replaced => "replaced",
                    },
                    "key": archive.key().as_path().display().to_string(),
                    "size_in_bytes": archive.size_in_bytes(),
                    "sha256": archive.sha256().as_str(),
                }),
                Err(error) => json!({
                    "period": period,
                    "failed_state": error.state_name,
                    "error": error.to_string(),
                }),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use storage::shared::Sha256Digest;

    use super::*;
    use crate::error::state_machine::state::FailedRequestExecution;
    use crate::implementations::states::extract::store_notes_archive::notes_key;
    use crate::prelude::StreamEvent;
    use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use crate::shared::status_code::StatusCode;

    /// Returns the archive of `period` with the bytes `PK`, under the key that `notes_key` gives.
    fn archive_of(period: &str) -> NotesArchiveRef {
        let period = NotesPeriod::new(period)
            .expect("Given a hardcoded published period, parsing should always succeed");
        NotesArchiveRef::new(period, notes_key(period), 2, Sha256Digest::of(b"PK"))
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, building it should always succeed")
    }

    fn failed(period: &str, source: StateMachine) -> (String, PeriodOutcome) {
        let error = StreamError {
            event: StreamEvent::StateFailed,
            execution_id: uuid::Uuid::nil(),
            state_name: "Execute SEC Request".to_string(),
            data: Value::Null,
            source,
        };
        (period.to_string(), Err(error))
    }

    fn stored(period: &str) -> (String, PeriodOutcome) {
        stored_by(period, Fetch::Downloaded)
    }

    fn stored_by(period: &str, fetch: Fetch) -> (String, PeriodOutcome) {
        (period.to_string(), Ok((fetch, archive_of(period))))
    }

    fn answered(period: &str, status_code: StatusCode) -> (String, PeriodOutcome) {
        let response_error =
            InvalidSecResponse::new(ErrorReason::InvalidStatusCode { status_code });
        let failure = FailedRequestExecution::new(
            "Execute SEC Request",
            FailedSecRequest::from(response_error),
        );
        failed(
            period,
            StateMachine::State(StateError::FailedRequestExecution(failure)),
        )
    }

    #[test]
    fn should_return_ok_when_only_the_last_period_is_absent() {
        let summary = NotesRunSummary(vec![
            stored("2026_08"),
            answered("2026_09", StatusCode::NotFound),
        ]);

        let expected_result = Ok(());

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_only_already_stored_periods_when_the_run_mixes_fetches() {
        let summary = NotesRunSummary(vec![
            stored_by("2009q1", Fetch::AlreadyStored),
            stored_by("2009q2", Fetch::Downloaded),
            stored_by("2009q3", Fetch::Replaced),
        ]);

        let expected_result = 1;

        let result = summary.already_stored_count();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_the_status_of_each_stored_period_when_listing_entries() {
        let summary = NotesRunSummary(vec![
            stored_by("2009q1", Fetch::AlreadyStored),
            stored_by("2009q2", Fetch::Downloaded),
            stored_by("2009q3", Fetch::Replaced),
        ]);

        let expected_result = vec![
            json!("already_stored"),
            json!("downloaded"),
            json!("replaced"),
        ];

        let result: Vec<Value> = summary
            .entries()
            .into_iter()
            .map(|entry| entry["status"].clone())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_absent_periods_as_failed_when_more_than_the_last_period_is_absent() {
        let summary = NotesRunSummary(vec![
            answered("2026_08", StatusCode::Forbidden),
            answered("2026_09", StatusCode::Forbidden),
        ]);

        let expected_result = Err(FailedBackfill {
            failures: 1,
            periods: 2,
        });

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_an_absent_period_as_failed_when_a_later_period_is_stored() {
        let summary = NotesRunSummary(vec![
            answered("2026_08", StatusCode::NotFound),
            stored("2026_09"),
        ]);

        let expected_result = Err(FailedBackfill {
            failures: 1,
            periods: 2,
        });

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_an_absent_period_as_failed_when_a_later_period_failed_otherwise() {
        let summary = NotesRunSummary(vec![
            answered("2026_09", StatusCode::NotFound),
            failed("2026_10", StateMachine::InvalidConfiguration),
        ]);

        let expected_result = Err(FailedBackfill {
            failures: 2,
            periods: 2,
        });

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_a_last_period_as_failed_when_the_sec_answers_500() {
        let summary = NotesRunSummary(vec![answered("2026_09", StatusCode::InternalServerError)]);

        let expected_result = Err(FailedBackfill {
            failures: 1,
            periods: 1,
        });

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_mark_the_last_absent_period_as_unpublished_when_listing_entries() {
        let summary = NotesRunSummary(vec![
            failed("2026_08", StateMachine::InvalidConfiguration),
            answered("2026_09", StatusCode::NotFound),
        ]);

        let expected_result = json!({ "period": "2026_09", "status": "unpublished" });

        let result = summary.entries()[1].clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_last_day_of_the_newest_period_when_every_period_is_stored() {
        let summary = NotesRunSummary(vec![
            stored("2025q3"),
            stored_by("2025_10", Fetch::AlreadyStored),
        ]);

        let expected_result = Ok(date(2025, 10, 31));

        let result = summary.cut_over_date();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_last_day_of_the_period_before_when_the_newest_period_is_unpublished() {
        let summary = NotesRunSummary(vec![
            stored("2025q3"),
            answered("2025_10", StatusCode::NotFound),
        ]);

        let expected_result = Ok(date(2025, 9, 30));

        let result = summary.cut_over_date();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_backfill_when_an_older_period_failed() {
        let summary = NotesRunSummary(vec![
            answered("2025q3", StatusCode::NotFound),
            stored("2025_10"),
        ]);

        let expected_result = Err(MissingCutOverDate::FailedBackfill(FailedBackfill {
            failures: 1,
            periods: 2,
        }));

        let result = summary.cut_over_date();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_stored_period_when_no_period_is_stored() {
        let summary = NotesRunSummary(vec![answered("2025_10", StatusCode::NotFound)]);

        let expected_result = Err(MissingCutOverDate::MissingStoredPeriod);

        let result = summary.cut_over_date();

        assert_eq!(result, expected_result);
    }
}
