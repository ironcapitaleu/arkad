//! # Batch Log
//!
//! Provides the startup steps of a batch binary that log their failure, and the observer and
//! summary logs of a facade run. Each call records its context as a JSON object, for the logger in
//! `json_log.rs`.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::time::Duration;

use chrono::NaiveDate;
use sec::implementations::extraction::{DailyRunSummary, ErrorKind, Unit};
use sec::prelude::{StreamError, StreamItem};
use sec::shared::http_client::implementations::sec_client::SecClient;
use serde_json::Value;
use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
use uuid::Uuid;

use crate::sec_client::{
    USER_AGENT_ENV_VAR, default_user_agent_warning, sec_client, startup_error_cause,
};
use crate::storage_root::create_root;

/// Top-level batch events.
// Only `backfill` logs the step events. `daily_filings` includes this file too, so `dead_code`
// would fire there.
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum BatchEvent {
    Complete,
    DefaultUserAgent,
    Failed,
    /// A request that asks whether a Notes file exists failed with a transient SEC failure and
    /// runs again.
    NotesProbeRetried,
    StartupFailed,
    /// One facade call of a run with two calls ended with no failure.
    StepComplete,
    /// One facade call of a run with two calls ended with a failure.
    StepFailed,
}

impl Display for BatchEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => write!(f, "batch_complete"),
            Self::DefaultUserAgent => write!(f, "batch_default_user_agent"),
            Self::Failed => write!(f, "batch_failed"),
            Self::NotesProbeRetried => write!(f, "batch_notes_probe_retried"),
            Self::StartupFailed => write!(f, "batch_startup_failed"),
            Self::StepComplete => write!(f, "batch_step_complete"),
            Self::StepFailed => write!(f, "batch_step_failed"),
        }
    }
}

/// Builds the SEC client of the run, logs a failure, and warns about the default `User-Agent`.
///
/// # Errors
///
/// Returns the error of [`sec_client`].
pub fn startup_sec_client() -> Result<SecClient, Box<dyn Error + Send + Sync>> {
    let sec_client = sec_client().inspect_err(|error| {
        let cause = startup_error_cause(error.as_ref());
        tracing::error!(
            event = %BatchEvent::StartupFailed,
            message = %format!(
                "[FailedSecClientCreation] Failed to build the SEC client, Caused by: {cause}"
            ),
            context = %serde_json::json!({ "env_var": USER_AGENT_ENV_VAR }),
        );
    })?;
    if let Some(warning) = default_user_agent_warning(&sec_client) {
        tracing::warn!(
            event = %BatchEvent::DefaultUserAgent,
            message = %warning,
            context = %serde_json::json!({ "env_var": USER_AGENT_ENV_VAR }),
        );
    }
    Ok(sec_client)
}

/// Creates the root directory of `repository`, and logs a failure.
///
/// # Errors
///
/// Returns the error of [`create_root`].
pub fn startup_create_root<D>(repository: &FilesystemRepository<Write, D>) -> std::io::Result<()> {
    create_root(repository).inspect_err(|error| {
        tracing::error!(
            event = %BatchEvent::StartupFailed,
            message = %format!(
                "[FailedStorageRootCreation] Failed to create the storage root, Reason: '{error}'"
            ),
            context = %serde_json::json!({ "root": repository.root().display().to_string() }),
        );
    })
}

/// Logs one successful stream item of the machine of `unit`.
pub fn log_item(execution_id: Uuid, unit: &Unit, item: &StreamItem) {
    tracing::info!(
        event = %item.event,
        message = %format!("{}: '{}'", item.event, item.state_name),
        context = %serde_json::json!({
            "execution_id": execution_id.to_string(),
            "run": unit,
            "state": item.state_name,
            "event_duration_ms": item.event_duration.as_millis(),
            "data": item.data,
        }),
    );
}

/// Logs one error of the machine of `unit`, at `error` level only if it fails the run.
///
/// The other kinds are expected on a healthy run: a transient SEC failure that a retry fixes, a
/// daily index that the SEC has not published yet, a filing without an XBRL instance, and a Notes
/// file that the SEC has not posted yet. They are `warn`.
pub fn log_error(execution_id: Uuid, unit: &Unit, error: &StreamError, kind: ErrorKind) {
    let message = error.source.to_string();
    let context = serde_json::json!({
        "execution_id": execution_id.to_string(),
        "run": unit,
        "kind": kind.to_string(),
        "state": error.state_name,
        "data": error.data,
    });
    if kind == ErrorKind::Failed {
        tracing::error!(event = %error.event, message = %message, context = %context);
    } else {
        tracing::warn!(event = %error.event, message = %message, context = %context);
    }
}

/// Returns the message and the context of the summary log of a run of the new-filing path.
pub fn daily_summary(
    execution_id: Uuid,
    dates: &[NaiveDate],
    summary: &DailyRunSummary,
    elapsed: Duration,
) -> (String, Value) {
    let context = serde_json::json!({
        "execution_id": execution_id.to_string(),
        "first_date": dates.first().map(ToString::to_string),
        "last_date": dates.last().map(ToString::to_string),
        "duration_ms": elapsed.as_millis(),
        "dates": summary.entries(),
    });
    let message = format!("Ran {} dates in {elapsed:.2?}", dates.len());
    (message, context)
}

/// Logs a summary: at `info` with `complete` if `result` is `Ok`, else at `error` with `failed`
/// and the error after `message`.
pub fn log_summary<E: Display>(
    result: &Result<(), E>,
    complete: BatchEvent,
    failed: BatchEvent,
    message: &str,
    context: &Value,
) {
    match result {
        Ok(()) => tracing::info!(event = %complete, message = %message, context = %context),
        Err(error) => {
            tracing::error!(event = %failed, message = %format!("{message}, {error}"), context = %context);
        }
    }
}
