mod extraction;

use extraction::Extraction;
use futures_util::StreamExt;

use extraction::constants::CIKS;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            Err(e) => {
                failures += 1;
                eprintln!("Error: {e}");
            }
        }
    }

    println!("\n{successes} succeeded, {failures} failed in {elapsed:.2?}");

    Ok(())
}
