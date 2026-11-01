mod run;
// Each binary is its own crate root, so it cannot import another binary's modules. `#[path]`
// compiles each shared file into this binary as a private module, so all binaries build the
// client and the store the same way.
#[path = "../shared/batch_log.rs"]
mod batch_log;
#[path = "../shared/files_in_flight.rs"]
mod files_in_flight;
#[path = "../shared/notes_periods.rs"]
mod notes_periods;
#[path = "../shared/sec_client.rs"]
mod sec_client;
// This binary uses only the Notes writer and the generic writer, so the Company Facts items would
// fire `dead_code`.
#[allow(dead_code)]
#[path = "../shared/storage_root.rs"]
mod storage_root;
#[path = "../shared/yesterday.rs"]
mod yesterday;

use std::cell::Cell;
use std::env;
use std::error::Error;
use std::time::{Duration, Instant};

use chrono::Utc;
use sec::implementations::extraction::{
    AllRunSummary, DailyFilingsRepositories, Extraction, FailedAllBackfill, NotesRunSummary,
    RunEvent,
};
use uuid::Uuid;

use batch_log::{
    BatchEvent, daily_summary, init_logger, log_error, log_item, log_summary, startup_create_root,
    startup_sec_client,
};
use files_in_flight::{FILES_IN_FLIGHT_ENV_VAR, InvalidFilesInFlight, files_in_flight};
use notes_periods::notes_periods;
use run::{BackfillInputs, run};
use storage_root::{raw_notes_archive_writer, raw_writer};
use yesterday::new_york_yesterday;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let _guard = init_logger();

    let sec_client = startup_sec_client()?;
    let files_in_flight = files_in_flight_from_env()?;
    let notes_repository = raw_notes_archive_writer();
    // Every writer shares the root of `ARKAD_STORAGE_ROOT`, so one call creates it.
    startup_create_root(&notes_repository)?;
    let daily_repositories = DailyFilingsRepositories {
        daily_index: raw_writer(),
        filing_index: raw_writer(),
        instance: raw_writer(),
        labels: raw_writer(),
        submissions: raw_writer(),
    };
    let backfill = Extraction::backfill();
    let execution_id = backfill.id();
    let now = Utc::now();
    let yesterday = new_york_yesterday(now);
    let inputs = BackfillInputs {
        backfill,
        periods: notes_periods(now.date_naive()),
        files_in_flight,
        yesterday,
        sec_client,
        notes_repository,
        daily_repositories,
    };
    let start = Instant::now();
    let catch_up_start = Cell::new(start);
    let mut past_result: Result<(), FailedAllBackfill> = Ok(());

    let catch_up = run(
        inputs,
        move |unit, event| match event {
            RunEvent::Item(item) => log_item(execution_id, unit, item),
            RunEvent::Error(error, kind) => log_error(execution_id, unit, error, kind),
        },
        |past| {
            log_notes_summary(execution_id, files_in_flight, past.notes(), start.elapsed());
            log_submissions_summary(execution_id, past);
            past_result = past.result();
            catch_up_start.set(Instant::now());
        },
    )
    .await
    .inspect_err(|error| {
        tracing::error!(
            event = %BatchEvent::Failed,
            message = %error,
            context = %serde_json::json!({ "execution_id": execution_id.to_string() }),
        );
    })?;

    let (message, mut context) = daily_summary(
        execution_id,
        &catch_up.dates,
        &catch_up.summary,
        catch_up_start.get().elapsed(),
    );
    context["step"] = serde_json::json!("catch_up");
    log_summary(
        &catch_up.summary.result(),
        BatchEvent::StepComplete,
        BatchEvent::StepFailed,
        &message,
        &context,
    );
    // A failed Notes period already stopped the run at `cut_over_date()`. So here only a failed CIK,
    // an unreadable Notes file, or the catch-up can fail.
    let failures: Vec<String> = [
        past_result.err().map(|error| error.to_string()),
        catch_up
            .summary
            .result()
            .err()
            .map(|error| error.to_string()),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !failures.is_empty() {
        let message = failures.join(", ");
        tracing::error!(
            event = %BatchEvent::Failed,
            message = %message,
            context = %serde_json::json!({ "execution_id": execution_id.to_string() }),
        );
        return Err(message.into());
    }
    tracing::info!(
        event = %BatchEvent::Complete,
        message = %format!("Ran the Notes periods, the submissions of their CIKs, and every date up to {yesterday} in {:.2?}", start.elapsed()),
        context = %serde_json::json!({ "execution_id": execution_id.to_string() }),
    );
    Ok(())
}

/// Logs the summary of the Notes periods, before the catch-up starts.
///
/// `elapsed` is the time of the first call, so it includes the `submissions` step.
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

/// Logs the summary of the `submissions` step, before the catch-up starts.
///
/// The step fails if a CIK failed or if a stored Notes file is unreadable.
fn log_submissions_summary(execution_id: Uuid, summary: &AllRunSummary) {
    let submissions = summary.submissions();
    let successes = submissions.success_count();
    let failures = submissions.failure_count();
    let unreadable = summary.unreadable_file_count();
    let context = serde_json::json!({
        "execution_id": execution_id.to_string(),
        "step": "submissions",
        "successes": successes,
        "failures": failures,
        "unreadable_files": unreadable,
        "files": summary.cik_read_entries(),
        // Only the failed CIKs: an entry per CIK makes the line longer than the 256 KB that Cloud
        // Logging accepts.
        "failed_ciks": submissions.failed_entries(),
    });
    let message = format!(
        "{successes} submissions files stored, {failures} failed, {unreadable} Notes files unreadable"
    );
    let result = match summary.result() {
        Err(error) if failures + unreadable > 0 => Err(error),
        _ => Ok(()),
    };
    log_summary(
        &result,
        BatchEvent::StepComplete,
        BatchEvent::StepFailed,
        &message,
        &context,
    );
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
