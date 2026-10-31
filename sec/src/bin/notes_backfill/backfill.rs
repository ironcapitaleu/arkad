//! # Backfill
//!
//! Provides [`run`], which extracts one Notes file per period, and the [`RunSummary`] it returns.
//!
//! The SEC posts the file of a month some days into the next month. So the newest periods of a run
//! can have no file yet. [`RunSummary`] counts the periods at the end of the run whose file is
//! absent as unpublished, not as failed. The next run asks for them again.

use std::panic;
use std::sync::Arc;

use futures_util::StreamExt;
use sec::error::{State as StateError, StateMachine};
use sec::implementations::extraction::Backfill;
use sec::prelude::{StreamError, StreamItem};
use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::notes_archive_ref::NotesArchiveRef;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::RawNotesArchive;
use serde_json::{Value, json};
use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
use thiserror::Error;

/// The most Notes files that the run downloads at once.
///
/// The rate limiter limits requests per second, not files in memory, so this cap bounds memory.
pub const FILES_IN_FLIGHT: usize = 2;

/// The outcome of one period: the stored archive, or the error of the state that failed.
pub type PeriodOutcome = Result<NotesArchiveRef, StreamError>;

/// The outcome of every period of one run, in the order of the given periods.
pub struct RunSummary(Vec<(String, PeriodOutcome)>);

/// Returns `true` if `error` comes from an SEC answer of 403 or 404 for the Notes file.
pub const fn is_absent_file(error: &StreamError) -> bool {
    matches!(
        &error.source,
        StateMachine::State(StateError::FailedRequestExecution(failure))
            if failure.domain_error().is_absent_file()
    )
}

/// The error of a run in which at least one period failed.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("[FailedBackfill] {failures} of {periods} Notes periods failed")]
pub struct FailedBackfill {
    /// The number of periods that failed.
    pub failures: usize,
    /// The number of periods in the run.
    pub periods: usize,
}

impl RunSummary {
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

    /// Returns the number of periods that stored their file.
    pub fn success_count(&self) -> usize {
        self.0.iter().filter(|(_, outcome)| outcome.is_ok()).count()
    }

    /// Returns the number of periods that failed and are not unpublished.
    pub fn failure_count(&self) -> usize {
        self.0.len() - self.success_count() - self.unpublished_count()
    }

    /// Returns the number of periods at the end of the run whose file is absent.
    pub fn unpublished_count(&self) -> usize {
        self.0
            .iter()
            .rev()
            .take_while(|(_, outcome)| outcome.as_ref().is_err_and(is_absent_file))
            .count()
    }

    /// Returns one JSON entry per period.
    ///
    /// A stored period lists its key, size, and SHA-256. An unpublished period says so. A failed
    /// period lists the failed state and the error.
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
                Ok(archive) => json!({
                    "period": period,
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

/// Extracts the Notes file of each period in `periods` into `repository`.
///
/// Every super-state comes from `backfill`, so all of them carry one `execution_id`. At most
/// [`FILES_IN_FLIGHT`] periods run at once, each in its own task. A failed period does not stop
/// the others.
///
/// Calls `on_item` with the period and each successful stream item, and `on_error` with the period
/// and the error of a failed period.
///
/// # Panics
///
/// Panics if the task of a period panics, with the payload of that panic.
pub async fn run(
    backfill: Backfill,
    periods: &[&str],
    sec_client: &SecClient,
    repository: &FilesystemRepository<Write, RawDocument<RawNotesArchive>>,
    on_item: impl Fn(&str, &StreamItem) + Send + Sync + 'static,
    on_error: impl Fn(&str, &StreamError) + Send + Sync + 'static,
) -> RunSummary {
    let on_item = Arc::new(on_item);
    let on_error = Arc::new(on_error);
    let extractions = periods.iter().enumerate().map(|(index, &period)| {
        let super_state = backfill
            .notes()
            .period(period)
            .sec_client(sec_client.clone())
            .repository(repository.clone())
            .build();
        let period = period.to_string();
        let on_item = Arc::clone(&on_item);
        let on_error = Arc::clone(&on_error);
        // STA-344 added a task per period after a stream cut-off. Its cause is open, so keeping
        // this split is an open decision (STA-360).
        async move {
            let task = tokio::spawn(async move {
                let outcome = super_state
                    .run_to_archive(|item| on_item(&period, item))
                    .await
                    .inspect_err(|error| on_error(&period, error));
                (index, period, outcome)
            });
            task.await
                .unwrap_or_else(|error| panic::resume_unwind(error.into_panic()))
        }
    });
    let mut outcomes: Vec<_> = futures_util::stream::iter(extractions)
        .buffer_unordered(FILES_IN_FLIGHT)
        .collect()
        .await;
    outcomes.sort_by_key(|(index, _, _)| *index);
    RunSummary(
        outcomes
            .into_iter()
            .map(|(_, period, outcome)| (period, outcome))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use sec::error::state_machine::state::FailedRequestExecution;
    use sec::prelude::StreamEvent;
    use sec::shared::http_client::implementations::sec_client::error::FailedSecRequest;
    use sec::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use sec::shared::status_code::StatusCode;

    use super::*;

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
    fn should_return_ok_when_only_the_last_periods_are_absent() {
        let summary = RunSummary(vec![
            answered("2026_09", StatusCode::NotFound),
            answered("2026_10", StatusCode::Forbidden),
        ]);

        let expected_result = Ok(());

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_an_absent_period_as_failed_when_a_later_period_failed_otherwise() {
        let summary = RunSummary(vec![
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
        let summary = RunSummary(vec![answered("2026_09", StatusCode::InternalServerError)]);

        let expected_result = Err(FailedBackfill {
            failures: 1,
            periods: 1,
        });

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_mark_the_last_absent_period_as_unpublished_when_listing_entries() {
        let summary = RunSummary(vec![
            failed("2026_08", StateMachine::InvalidConfiguration),
            answered("2026_09", StatusCode::NotFound),
        ]);

        let expected_result = json!({ "period": "2026_09", "status": "unpublished" });

        let result = summary.entries()[1].clone();

        assert_eq!(result, expected_result);
    }
}
