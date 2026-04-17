//! # Pipeline Coverage Integration Test
//!
//! Runs the full Extract → Transform pipeline against SEC EDGAR to measure
//! pipeline quality and track progress as XBRL aliases are expanded.
//!
//! ## Running
//! ```sh
//! cargo test --test pipeline_coverage -- --ignored
//! ```

mod builder;
mod constants;

use futures_util::StreamExt;

use builder::Pipeline;
use constants::{MUST_PASS_CIKS, SP500_CIKS};

/// Looks up the company name for a CIK in the must-pass list.
///
/// This is used to provide the company name in the test failure message since the CIK alone is not human-friendly.
/// Returns "Unknown" for the company name if the CIK is not in the list (should never happen in this test).
fn lookup_must_pass_name(cik: &str) -> &str {
    MUST_PASS_CIKS
        .iter()
        .find(|(c, _)| *c == cik)
        .map_or("Unknown", |(_, name)| name)
}

/// Minimum number of S&P 500 companies that must succeed.
/// Increase this threshold as XBRL aliases are expanded.
const MINIMUM_SUCCESS_THRESHOLD: usize = 270;

/// Runs all CIKs through the pipeline concurrently and returns (successes, failures with details).
async fn run_batch(ciks: &[&str]) -> (usize, Vec<(String, String)>) {
    let results: Vec<_> = futures_util::stream::iter(ciks.iter())
        .map(|cik| Pipeline::builder().cik(*cik).build().run())
        .buffer_unordered(10)
        .collect()
        .await;

    let mut successes = 0;
    let mut failures = Vec::new();

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(()) => successes += 1,
            Err(e) => failures.push((ciks[i].to_string(), e.source.to_string())),
        }
    }

    (successes, failures)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "Hits the live SEC API for must-pass companies (~10 seconds)"]
async fn should_succeed_for_all_must_pass_companies() {
    let ciks: Vec<&str> = MUST_PASS_CIKS.iter().map(|(cik, _)| *cik).collect();

    let expected_result: Vec<String> = vec![];

    let (_successes, failures) = run_batch(&ciks).await;
    let result: Vec<String> = failures
        .iter()
        .map(|(cik, msg)| {
            let name = lookup_must_pass_name(cik);
            format!("{name} (CIK {cik}): {msg}")
        })
        .collect();

    assert_eq!(result, expected_result);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "Hits the live SEC API for all S&P 500 CIKs (~3 minutes)"]
async fn should_pass_minimum_threshold_for_sp500_companies() {
    let ciks: Vec<&str> = SP500_CIKS.to_vec();

    let expected_result = true;

    let (successes, failures) = run_batch(&ciks).await;

    println!("\n============================================================");
    println!(
        "Pipeline Coverage: {successes}/{} passed ({} failed)",
        ciks.len(),
        failures.len()
    );
    println!("============================================================");
    if !failures.is_empty() {
        println!("\nFailed CIKs:");
        for (cik, msg) in &failures {
            println!("  CIK {cik}: {msg}");
        }
    }
    println!("\nThreshold: {successes} >= {MINIMUM_SUCCESS_THRESHOLD}");

    let result = successes >= MINIMUM_SUCCESS_THRESHOLD;

    assert_eq!(result, expected_result);
}
