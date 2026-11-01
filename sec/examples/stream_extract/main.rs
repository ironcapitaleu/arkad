//! Deprecated example: extracts the `companyfacts` JSON of a fixed list of S&P 500 CIKs.
//!
//! It runs `Extraction::backfill().company_facts()` against the live SEC and stores each response
//! under `ARKAD_STORAGE_ROOT`. The `Dockerfile` builds and runs it. See "Deprecated Binaries" in
//! `sec/design/pipeline/ingestion_paths.md`.
//!
//! Run it with `cargo run -p sec --features tracing-logging --example stream_extract`.

mod constants;
// Each example is its own crate root, so it cannot import a binary's modules. `#[path]` compiles
// each shared file into this example as a private module, so the example builds the client and the
// store as the binaries do.
#[path = "../../src/bin/shared/sec_client.rs"]
mod sec_client;
#[path = "../../src/bin/shared/storage_root.rs"]
mod storage_root;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::stdout;
use std::time::Instant;

use sec::implementations::extraction::{Extraction, RunEvent};
use tracing_subscriber::fmt::format::FmtSpan;

use constants::CIKS;
use sec_client::{USER_AGENT_ENV_VAR, default_user_agent_warning, sec_client, startup_error_cause};
use storage_root::{create_root, raw_company_facts_writer};

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
    // Initialize non-blocking JSON structured logging
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
            env_var = USER_AGENT_ENV_VAR,
        );
    })?;
    if let Some(warning) = default_user_agent_warning(&sec_client) {
        tracing::warn!(
            event = %BatchEvent::DefaultUserAgent,
            message = %warning,
            env_var = USER_AGENT_ENV_VAR,
        );
    }
    let repository = raw_company_facts_writer();
    create_root(&repository).inspect_err(|error| {
        tracing::error!(
            event = %BatchEvent::StartupFailed,
            message = %format!("Failed to create the storage root, Caused by: {error}"),
            root = %repository.root().display(),
        );
    })?;
    let backfill = Extraction::backfill();
    let execution_id = backfill.id();
    let start = Instant::now();

    let summary = backfill
        .company_facts()
        .ciks(CIKS)
        .sec_client(sec_client)
        .repository(repository)
        // All CIKs at once, as before the facade. The rate limiter of the client caps the requests.
        .in_flight(CIKS.len())
        .run(move |unit, event| match event {
            RunEvent::Item(item) => {
                tracing::info!(
                    event = %item.event,
                    message = %format!("{}: '{}'", item.event, item.state_name),
                    event_duration_ms = item.event_duration.as_millis(),
                    context = %serde_json::json!({
                        "execution_id": execution_id.to_string(),
                        "cik": unit.to_string(),
                        "state": item.state_name,
                        "data": item.data,
                    }),
                );
            }
            RunEvent::Error(error, _) => {
                tracing::error!(
                    event = %error.event,
                    message = %error.source.to_string(),
                    context = %serde_json::json!({
                        "execution_id": error.execution_id.to_string(),
                        "cik": unit.to_string(),
                        "state": error.state_name,
                        "data": error.data,
                    }),
                );
            }
        })
        .await;

    let elapsed = start.elapsed();
    let successes = summary.success_count();
    let failures = summary.failure_count();
    let message = format!("{successes} succeeded, {failures} failed in {elapsed:.2?}");
    let context = serde_json::json!({
        "execution_id": execution_id.to_string(),
        "successes": successes,
        "failures": failures,
        "duration_ms": elapsed.as_millis(),
        "ciks": summary.entries(),
    });
    summary.result().inspect_err(|_| {
        tracing::error!(event = %BatchEvent::Failed, message = %message, context = %context);
    })?;
    tracing::info!(event = %BatchEvent::Complete, message = %message, context = %context);
    Ok(())
}
