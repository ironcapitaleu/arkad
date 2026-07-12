mod pipeline;

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::stdout;
use std::time::Instant;

use pipeline::Pipeline;
use sec::shared::http_client::implementations::sec_client::SecClient;
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

    let sec_client = SecClient::default();
    let start = Instant::now();

    // Concurrency is intentionally unbounded: the 10 req/s cap is enforced across all clones of
    // this SecClient instance (Arc-backed limiter), so futures simply park at the permit gate
    // (natural backpressure) rather than being throttled by a bounded concurrency window.
    // If the CIK count grows to thousands, revisit with buffer_unordered to cap in-flight futures.
    let pipelines = CIKS.into_iter().map(|cik| {
        Pipeline::builder()
            .cik(cik)
            .sec_client(sec_client.clone())
            .build()
            .run()
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
        successes = successes,
        failures = failures,
        duration_ms = elapsed.as_millis(),
    );

    Ok(())
}
