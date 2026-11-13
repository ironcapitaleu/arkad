// Each binary is its own crate root, so it cannot import another binary's modules. `#[path]`
// compiles each shared file into this binary as a private module, so all binaries build the
// client and the store the same way.
#[path = "../shared/batch_log.rs"]
mod batch_log;
#[path = "../shared/files_in_flight.rs"]
mod files_in_flight;
#[path = "../shared/json_log.rs"]
mod json_log;
#[path = "../shared/new_york_clock.rs"]
mod new_york_clock;
#[path = "../shared/notes_periods.rs"]
mod notes_periods;
#[path = "../shared/sec_client.rs"]
mod sec_client;
// This binary uses only the Notes writer and the generic writer, so the Company Facts items would
// fire `dead_code`.
#[allow(dead_code)]
#[path = "../shared/storage_root.rs"]
mod storage_root;

use std::env;
use std::error::Error;
use std::fmt::Display;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use chrono::{NaiveDate, Utc};
use sec::implementations::extraction::{
    BackfillRepositories, DailyFilingsRepositories, Extraction, FailedNotesProbe, NotesRunSummary,
    RunEvent,
};
use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use sec::shared::notes_period::NotesPeriod;
use uuid::Uuid;

use batch_log::{
    BatchEvent, daily_summary, log_error, log_item, log_summary, startup_create_root,
    startup_sec_client,
};
use files_in_flight::{FILES_IN_FLIGHT_ENV_VAR, InvalidFilesInFlight, files_in_flight};
use json_log::init_logger;
use new_york_clock::previous_day_in_new_york;
use notes_periods::notes_periods;
use storage_root::{raw_notes_archive_writer, raw_writer};

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
    let sec_client = startup_sec_client()?;
    let files_in_flight = files_in_flight_from_env()?;
    let notes_repository = raw_notes_archive_writer();
    // Every writer shares the root of `ARKAD_STORAGE_ROOT`, so one call creates it.
    startup_create_root(&notes_repository)?;
    let repositories = BackfillRepositories {
        notes: notes_repository,
        daily_filings: DailyFilingsRepositories {
            daily_index: raw_writer(),
            filing_index: raw_writer(),
            instance: raw_writer(),
            labels: raw_writer(),
            submissions: raw_writer(),
        },
    };
    let backfill = Extraction::backfill();
    let execution_id = backfill.id();
    let now = Utc::now();
    let last_date = previous_day_in_new_york(now);
    let periods = notes_periods_of_run(&sec_client, now.date_naive()).await?;
    let start = Instant::now();

    let summary = backfill
        .all()
        .periods(periods)
        .until(last_date)
        .sec_client(sec_client)
        .repositories(repositories)
        .in_flight(files_in_flight)
        .run(move |unit, event| match event {
            RunEvent::Item(item) => log_item(execution_id, unit, item),
            RunEvent::Error(error, kind) => log_error(execution_id, unit, error, kind),
        })
        .await
        .inspect_err(|error| log_failed(execution_id, error))?;

    log_notes_summary(
        execution_id,
        files_in_flight,
        summary.notes(),
        summary.notes_duration(),
    );
    let (message, mut context) = daily_summary(
        execution_id,
        summary.catch_up_dates(),
        summary.catch_up(),
        summary.catch_up_duration(),
    );
    context["step"] = serde_json::json!("catch_up");
    log_summary(
        &summary.catch_up().result(),
        BatchEvent::StepComplete,
        BatchEvent::StepFailed,
        &message,
        &context,
    );
    summary
        .result()
        .inspect_err(|error| log_failed(execution_id, error))?;
    tracing::info!(
        event = %BatchEvent::Complete,
        message = %format!("Ran the Notes periods and every date up to {last_date} in {:.2?}", start.elapsed()),
        context = %serde_json::json!({ "execution_id": execution_id.to_string() }),
    );
    Ok(())
}

/// Logs `error` as the failure of the batch.
fn log_failed(execution_id: Uuid, error: &impl Display) {
    tracing::error!(
        event = %BatchEvent::Failed,
        message = %error,
        context = %serde_json::json!({ "execution_id": execution_id.to_string() }),
    );
}

/// Logs the summary of the Notes periods.
fn log_notes_summary(
    execution_id: Uuid,
    files_in_flight: usize,
    summary: &NotesRunSummary,
    elapsed: Duration,
) {
    let successes = summary.success_count();
    let already_stored = summary.already_stored_count();
    let failures = summary.failure_count();
    let unpublished = summary.unpublished_count();
    let context = serde_json::json!({
        "execution_id": execution_id.to_string(),
        "step": "notes",
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
    log_summary(
        &summary.result(),
        BatchEvent::StepComplete,
        BatchEvent::StepFailed,
        &message,
        &context,
    );
}

/// Lists the Notes periods of the run, and logs each retry and a failure.
///
/// # Errors
///
/// Returns the [`FailedNotesProbe`] of [`notes_periods`].
async fn notes_periods_of_run(
    sec_client: &SecClient,
    today: NaiveDate,
) -> Result<Vec<String>, FailedNotesProbe> {
    let on_retry = |quarter: NotesPeriod, error: &FailedSecRequest| {
        tracing::warn!(
            event = %BatchEvent::NotesProbeRetried,
            message = %error,
            context = %serde_json::json!({ "period": quarter.to_string() }),
        );
    };
    notes_periods(sec_client, today, on_retry)
        .await
        .inspect_err(|error| {
            tracing::error!(
                event = %BatchEvent::StartupFailed,
                message = %error,
                context = %serde_json::json!({ "period": error.period.to_string() }),
            );
        })
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
