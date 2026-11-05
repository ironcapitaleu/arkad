//! Deprecated example: extracts every Notes period up to last month.
//!
//! It runs `Extraction::backfill().notes()` against the live SEC and stores each file under
//! `ARKAD_STORAGE_ROOT`. The binary `backfill` replaces it. See "Deprecated Binaries" in
//! `sec/design/pipeline/ingestion_paths.md`.
//!
//! Run it with `cargo run -p sec --features tracing-logging --example notes_backfill`.

// Each example is its own crate root, so it cannot import a binary's modules. `#[path]` compiles
// each shared file into this example as a private module, as the binaries do.
#[path = "../../src/bin/shared/files_in_flight.rs"]
mod files_in_flight;
#[path = "../../src/bin/shared/notes_periods.rs"]
mod periods;
#[path = "../../src/bin/shared/sec_client.rs"]
mod sec_client;
// This example uses only the Notes writer, so the Company Facts items would fire `dead_code`.
#[allow(dead_code)]
#[path = "../../src/bin/shared/storage_root.rs"]
mod storage_root;

use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::stdout;
use std::time::Instant;

use chrono::Utc;
use sec::implementations::extraction::{ErrorKind, Extraction, RunEvent};
use tracing_subscriber::fmt::format::FmtSpan;

use files_in_flight::{FILES_IN_FLIGHT_ENV_VAR, InvalidFilesInFlight, files_in_flight};
use periods::notes_periods;
use sec_client::{USER_AGENT_ENV_VAR, default_user_agent_warning, sec_client, startup_error_cause};
use storage_root::{create_root, raw_notes_archive_writer};

// This deprecated example stays frozen, so it keeps its own copy of the log code in `batch_log.rs`.
/// Top-level batch events.
enum BatchEvent {
    Complete,
    DefaultUserAgent,
    Failed,
    StartupFailed,
}

impl Display for BatchEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => write!(f, "batch_complete"),
            Self::DefaultUserAgent => write!(f, "batch_default_user_agent"),
            Self::Failed => write!(f, "batch_failed"),
            Self::StartupFailed => write!(f, "batch_startup_failed"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(stdout());
    tracing_subscriber::fmt()
        .json()
        .with_span_events(FmtSpan::NONE)
        .with_target(false)
        .with_current_span(false)
        .flatten_event(true)
        .with_writer(non_blocking)
        .init();

    let sec_client = sec_client().inspect_err(|error| {
        let cause = startup_error_cause(error.as_ref());
        tracing::error!(
            event = %BatchEvent::StartupFailed,
            message = %format!("Failed to build the SEC client, Caused by: {cause}"),
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
    let files_in_flight = files_in_flight_from_env()?;
    let repository = raw_notes_archive_writer();
    create_root(&repository).inspect_err(|error| {
        tracing::error!(
            event = %BatchEvent::StartupFailed,
            message = %format!("Failed to create the storage root, Caused by: {error}"),
            context = %serde_json::json!({ "root": repository.root().display().to_string() }),
        );
    })?;
    let backfill = Extraction::backfill();
    let execution_id = backfill.id().to_string();
    let event_execution_id = execution_id.clone();
    let periods = notes_periods(Utc::now().date_naive());
    let start = Instant::now();

    let summary = backfill
        .notes()
        .periods(periods)
        .sec_client(sec_client)
        .repository(repository)
        .in_flight(files_in_flight)
        .run(move |unit, event| match event {
            RunEvent::Item(item) => {
                tracing::info!(
                    event = %item.event,
                    message = %format!("{}: '{}'", item.event, item.state_name),
                    context = %serde_json::json!({
                        "execution_id": event_execution_id,
                        "period": unit.to_string(),
                        "state": item.state_name,
                        "event_duration_ms": item.event_duration.as_millis(),
                        "data": item.data,
                    }),
                );
            }
            RunEvent::Error(error, kind) => {
                let message = error.source.to_string();
                let context = serde_json::json!({
                    "execution_id": error.execution_id.to_string(),
                    "period": unit.to_string(),
                    "state": error.state_name,
                    "data": error.data,
                });
                // The run summary decides whether an absent file counts as a failure.
                if kind == ErrorKind::Failed {
                    tracing::error!(event = %error.event, message = %message, context = %context);
                } else {
                    tracing::warn!(event = %error.event, message = %message, context = %context);
                }
            }
        })
        .await;

    let elapsed = start.elapsed();
    let failures = summary.failure_count();
    let successes = summary.success_count();
    let already_stored = summary.already_stored_count();
    let unpublished = summary.unpublished_count();
    let context = serde_json::json!({
        "execution_id": execution_id,
        "files_in_flight": files_in_flight,
        "successes": successes,
        "already_stored": already_stored,
        "failures": failures,
        "unpublished": unpublished,
        "duration_ms": elapsed.as_millis(),
        "periods": summary.entries(),
    });
    let message = format!(
        "{successes} Notes files stored ({already_stored} already stored), {failures} failed, {unpublished} unpublished in {elapsed:.2?}"
    );
    summary.result().inspect_err(|_| {
        tracing::error!(event = %BatchEvent::Failed, message = %message, context = %context);
    })?;
    tracing::info!(event = %BatchEvent::Complete, message = %message, context = %context);
    Ok(())
}

/// Reads the number of Notes files in flight from `ARKAD_FILES_IN_FLIGHT`, and logs an invalid value.
fn files_in_flight_from_env() -> Result<usize, InvalidFilesInFlight> {
    files_in_flight(env::var_os(FILES_IN_FLIGHT_ENV_VAR)).inspect_err(|error| {
        tracing::error!(
            event = %BatchEvent::StartupFailed,
            message = %error,
            context = %serde_json::json!({ "env_var": FILES_IN_FLIGHT_ENV_VAR }),
        );
    })
}
