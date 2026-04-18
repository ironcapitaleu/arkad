mod pipeline;

use std::fmt;

use futures_util::StreamExt;
use pipeline::Pipeline;
use tracing_subscriber::fmt::format::FmtSpan;

use pipeline::constants::CIKS;

/// Top-level batch events.
enum BatchEvent {
    Complete,
}

impl fmt::Display for BatchEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => write!(f, "batch_complete"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize non-blocking JSON structured logging
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::fmt()
        .json()
        .with_span_events(FmtSpan::NONE)
        .with_target(false)
        .with_current_span(false)
        .flatten_event(true)
        .with_writer(non_blocking)
        .init();

    let start = std::time::Instant::now();

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
