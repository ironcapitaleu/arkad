//! # Backfill
//!
//! Provides [`run`], which extracts one Notes file per period, and the [`RunSummary`] it returns.

use std::panic;
use std::sync::Arc;

use futures_util::StreamExt;
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
    /// Returns `Ok` if every period stored its file.
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
        self.0.len() - self.failure_count()
    }

    /// Returns the number of periods that failed.
    pub fn failure_count(&self) -> usize {
        self.0
            .iter()
            .filter(|(_, outcome)| outcome.is_err())
            .count()
    }

    /// Returns one JSON entry per period.
    ///
    /// A stored period lists its key, size, and SHA-256. A failed period lists the failed state and
    /// the error.
    pub fn entries(&self) -> Vec<Value> {
        self.0
            .iter()
            .map(|(period, outcome)| match outcome {
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
