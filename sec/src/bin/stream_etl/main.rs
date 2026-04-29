mod pipeline;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::stdout;
use std::time::Instant;

use futures_util::StreamExt;
use pipeline::Pipeline;
use tracing_subscriber::fmt::format::FmtSpan;

use pipeline::constants::CIKS;

/// Top-level batch events.
enum BatchEvent {
    Complete,
}

impl Display for BatchEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => write!(f, "batch_complete"),
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

    let start = Instant::now();

    let results: Vec<_> = futures_util::stream::iter(CIKS)
        .map(|cik| Pipeline::builder().cik(cik).build().run())
        .buffer_unordered(3)
        .collect()
        .await;

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
    );

    Ok(())
}
