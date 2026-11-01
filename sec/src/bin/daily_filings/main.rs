mod constants;
mod dates;
// Each binary is its own crate root, so it cannot import another binary's modules. `#[path]`
// compiles each shared file into this binary as a private module, so all binaries build the
// client and the store the same way.
#[path = "../shared/batch_log.rs"]
mod batch_log;
#[path = "../shared/sec_client.rs"]
mod sec_client;
// This binary uses only the generic writer, so the Company Facts and Notes items would fire
// `dead_code`.
#[allow(dead_code)]
#[path = "../shared/storage_root.rs"]
mod storage_root;
#[path = "../shared/yesterday.rs"]
mod yesterday;

use std::env;
use std::error::Error;
use std::time::Instant;

use chrono::Utc;
use sec::implementations::extraction::{DailyFilingsRepositories, Extraction, RunEvent};

use batch_log::{
    BatchEvent, daily_summary, init_logger, log_error, log_item, log_summary, startup_create_root,
    startup_sec_client,
};
use constants::{FROM_ENV_VAR, TO_ENV_VAR};
use dates::run_dates;
use storage_root::raw_writer;
use yesterday::new_york_yesterday;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let _guard = init_logger();

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
    let sec_client = startup_sec_client()?;
    let repositories = DailyFilingsRepositories {
        daily_index: raw_writer(),
        filing_index: raw_writer(),
        instance: raw_writer(),
        labels: raw_writer(),
        submissions: raw_writer(),
    };
    // All five writers share the root of `ARKAD_STORAGE_ROOT`, so one call creates it.
    startup_create_root(&repositories.daily_index)?;
    let incremental = Extraction::incremental();
    let execution_id = incremental.id();
    let start = Instant::now();

    let summary = incremental
        .daily_filings()
        .dates(dates.iter().copied())
        .yesterday(new_york_yesterday(now))
        .sec_client(sec_client)
        .repositories(repositories)
        .run(move |unit, event| match event {
            RunEvent::Item(item) => log_item(execution_id, unit, item),
            RunEvent::Error(error, kind) => log_error(execution_id, unit, error, kind),
        })
        .await;

    let (message, context) = daily_summary(execution_id, &dates, &summary, start.elapsed());
    let result = summary.result();
    log_summary(
        &result,
        BatchEvent::Complete,
        BatchEvent::Failed,
        &message,
        &context,
    );
    result?;
    Ok(())
}
