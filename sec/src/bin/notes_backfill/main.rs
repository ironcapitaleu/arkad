mod backfill;
mod constants;
mod files_in_flight;
mod periods;
// Each binary is its own crate root, so it cannot import another binary's modules. `#[path]`
// compiles each shared file into this binary as a private module, so all binaries build the
// client and the store the same way.
#[path = "../shared/sec_client.rs"]
mod sec_client;
// This binary uses only the Notes writer, so the Company Facts items would fire `dead_code`.
#[allow(dead_code)]
#[path = "../shared/storage_root.rs"]
mod storage_root;

use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io::stdout;
use std::time::Instant;

use chrono::Utc;
use sec::implementations::extraction::Extraction;
use tracing_subscriber::fmt::format::FmtSpan;

use constants::FILES_IN_FLIGHT_ENV_VAR;
use files_in_flight::{InvalidFilesInFlight, files_in_flight};
use periods::notes_periods;
use sec_client::{USER_AGENT_ENV_VAR, default_user_agent_warning, sec_client, startup_error_cause};
use storage_root::raw_notes_archive_writer;

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
    fs::create_dir_all(repository.root()).inspect_err(|error| {
        tracing::error!(
            event = %BatchEvent::StartupFailed,
            message = %format!("Failed to create the storage root, Caused by: {error}"),
            context = %serde_json::json!({ "root": repository.root().display().to_string() }),
        );
    })?;
    let backfill = Extraction::backfill();
    let execution_id = backfill.id().to_string();
    let item_execution_id = execution_id.clone();
    let periods = notes_periods(Utc::now().date_naive());
    let periods: Vec<&str> = periods.iter().map(String::as_str).collect();
    let start = Instant::now();

    let summary = backfill::run(
        backfill,
        &periods,
        files_in_flight,
        &sec_client,
        &repository,
        move |period, item| {
            tracing::info!(
                event = %item.event,
                message = %format!("{}: '{}'", item.event, item.state_name),
                context = %serde_json::json!({
                    "execution_id": item_execution_id,
                    "period": period,
                    "state": item.state_name,
                    "event_duration_ms": item.event_duration.as_millis(),
                    "data": item.data,
                }),
            );
        },
        |period, error| {
            let message = error.source.to_string();
            let context = serde_json::json!({
                "execution_id": error.execution_id.to_string(),
                "period": period,
                "state": error.state_name,
                "data": error.data,
            });
            // The run summary decides whether an absent file counts as a failure.
            if backfill::is_absent_file(error) {
                tracing::warn!(event = %error.event, message = %message, context = %context);
            } else {
                tracing::error!(event = %error.event, message = %message, context = %context);
            }
        },
    )
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
