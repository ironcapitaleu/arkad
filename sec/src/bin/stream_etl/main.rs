mod pipeline;
// Each binary is its own crate root, so it cannot import another binary's modules. `#[path]`
// compiles each shared file into this binary as a private module, so all binaries build the
// client and the store the same way.
#[path = "../shared/json_log.rs"]
mod json_log;
#[path = "../shared/sec_client.rs"]
mod sec_client;
#[path = "../shared/storage_root.rs"]
mod storage_root;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::process::ExitCode;
use std::time::Instant;

use pipeline::Pipeline;
use sec::implementations::extraction::Extraction;

use json_log::init_logger;
use sec_client::{USER_AGENT_ENV_VAR, default_user_agent_warning, sec_client, startup_error_cause};
use storage_root::{create_root, raw_company_facts_writer};

use pipeline::constants::CIKS;

/// Top-level batch events.
enum BatchEvent {
    Complete,
    DefaultUserAgent,
    StartupFailed,
}

impl Display for BatchEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => write!(f, "batch_complete"),
            Self::DefaultUserAgent => write!(f, "batch_default_user_agent"),
            Self::StartupFailed => write!(f, "batch_startup_failed"),
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let _guard = init_logger();
    // `run_batch` logs each failure where it happens. A `main` that returns an error prints a second
    // line in `Debug` form, which is not JSON.
    if run_batch().await.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Runs the batch, and logs each failure before it returns the error.
///
/// # Errors
///
/// Returns the first error that stops the batch.
async fn run_batch() -> Result<(), Box<dyn Error + Send + Sync>> {
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
    let repository = raw_company_facts_writer();
    create_root(&repository).inspect_err(|error| {
        tracing::error!(
            event = %BatchEvent::StartupFailed,
            message = %format!(
                "[FailedStorageRootCreation] Failed to create the storage root, Reason: '{error}'"
            ),
            context = %serde_json::json!({ "root": repository.root().display().to_string() }),
        );
    })?;
    // One handle, so every CIK of the run carries the same `execution_id`.
    let backfill = Extraction::backfill();
    let start = Instant::now();

    // Concurrency is intentionally unbounded: the 10 req/s cap is enforced by a process-global
    // rate limiter, so futures simply park at the permit gate (natural backpressure) rather than
    // being throttled by a bounded concurrency window.
    // If the CIK count grows to thousands, revisit with buffer_unordered to cap in-flight futures.
    let pipelines = CIKS.into_iter().map(|cik| {
        Pipeline::builder()
            .cik(cik)
            .sec_client(sec_client.clone())
            .repository(repository.clone())
            .build()
            .run(&backfill)
    });
    let results: Vec<_> = futures_util::future::join_all(pipelines).await;

    let elapsed = start.elapsed();

    let mut successes = 0;
    let mut failures = 0;
    for result in &results {
        match result {
            Ok(()) => successes += 1,
            Err(_) => failures += 1,
        }
    }

    tracing::info!(
        event = %BatchEvent::Complete,
        message = %format!("{successes} succeeded, {failures} failed in {elapsed:.2?}"),
        context = %serde_json::json!({
            "successes": successes,
            "failures": failures,
            "duration_ms": elapsed.as_millis(),
        }),
    );

    Ok(())
}
