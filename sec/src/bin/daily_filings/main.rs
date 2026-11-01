mod constants;
mod dates;
mod run;
// Each binary is its own crate root, so it cannot import another binary's modules. `#[path]`
// compiles each shared file into this binary as a private module, so all binaries build the
// client and the store the same way.
#[path = "../shared/sec_client.rs"]
mod sec_client;
// This binary uses only the generic writer, so the Company Facts and Notes items would fire
// `dead_code`.
#[allow(dead_code)]
#[path = "../shared/storage_root.rs"]
mod storage_root;

use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::stdout;
use std::time::Instant;

use chrono::Utc;
use sec::prelude::{StreamError, StreamItem};
use serde_json::Value;
use tracing_subscriber::fmt::format::FmtSpan;
use uuid::Uuid;

use constants::{FROM_ENV_VAR, RETRY_DELAYS, TO_ENV_VAR};
use dates::{new_york_yesterday, run_dates};
use run::{ErrorKind, Repositories, Settings};
use sec_client::{USER_AGENT_ENV_VAR, default_user_agent_warning, sec_client, startup_error_cause};
use storage_root::{create_root, raw_writer};

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

    let now = Utc::now();
    let dates = run_dates(env::var_os(FROM_ENV_VAR), env::var_os(TO_ENV_VAR), now).inspect_err(
        |error| {
            tracing::error!(
                event = %BatchEvent::StartupFailed,
                message = %error,
                context = %serde_json::json!({ "env_vars": [FROM_ENV_VAR, TO_ENV_VAR] }),
            );
        },
    )?;
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
    let repositories = Repositories {
        daily_index: raw_writer(),
        instance: raw_writer(),
        labels: raw_writer(),
        submissions: raw_writer(),
    };
    // All four writers share the root of `ARKAD_STORAGE_ROOT`, so one call creates it.
    create_root(&repositories.daily_index).inspect_err(|error| {
        tracing::error!(
            event = %BatchEvent::StartupFailed,
            message = %format!("Failed to create the storage root, Caused by: {error}"),
            context = %serde_json::json!({
                "root": repositories.daily_index.root().display().to_string(),
            }),
        );
    })?;
    let execution_id = Uuid::new_v4();
    let settings = Settings {
        execution_id,
        yesterday: new_york_yesterday(now),
        retry_delays: RETRY_DELAYS.to_vec(),
    };
    let start = Instant::now();

    let summary = run::run(
        &dates,
        &sec_client,
        &repositories,
        &settings,
        move |context, item| log_item(execution_id, context, item),
        move |context, error, kind| log_error(execution_id, context, error, kind),
    )
    .await;

    let elapsed = start.elapsed();
    let context = serde_json::json!({
        "execution_id": execution_id.to_string(),
        "first_date": dates.first().map(ToString::to_string),
        "last_date": dates.last().map(ToString::to_string),
        "duration_ms": elapsed.as_millis(),
        "dates": summary.entries(),
    });
    let message = format!("Ran {} dates in {elapsed:.2?}", dates.len());
    summary.result().inspect_err(|error| {
        tracing::error!(event = %BatchEvent::Failed, message = %format!("{message}, {error}"), context = %context);
    })?;
    tracing::info!(event = %BatchEvent::Complete, message = %message, context = %context);
    Ok(())
}

/// Logs one successful stream item of the machine that `context` names.
fn log_item(execution_id: Uuid, context: &Value, item: &StreamItem) {
    tracing::info!(
        event = %item.event,
        message = %format!("{}: '{}'", item.event, item.state_name),
        context = %serde_json::json!({
            "execution_id": execution_id.to_string(),
            "run": context,
            "state": item.state_name,
            "event_duration_ms": item.event_duration.as_millis(),
            "data": item.data,
        }),
    );
}

/// Logs one error of the machine that `context` names, at `error` level only if it fails the run.
///
/// The other kinds are expected on a healthy run: a 503 that a retry fixes, a daily index that the
/// SEC has not published yet, and a filing without an XBRL instance. They are `warn`.
fn log_error(execution_id: Uuid, context: &Value, error: &StreamError, kind: ErrorKind) {
    let message = error.source.to_string();
    let context = serde_json::json!({
        "execution_id": execution_id.to_string(),
        "run": context,
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
