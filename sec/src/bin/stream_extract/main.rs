mod extraction;

use extraction::Extraction;
use futures_util::StreamExt;
use tracing_subscriber::fmt::format::FmtSpan;

use extraction::constants::CIKS;

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
        .map(|cik| Extraction::builder().cik(cik).build().run())
        .buffer_unordered(10)
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
        event = "extraction_complete",
        message = %format!("{successes} succeeded, {failures} failed in {elapsed:.2?}"),
    );

    Ok(())
}
