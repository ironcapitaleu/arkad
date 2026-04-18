//! # Pipeline Coverage Integration Test
//!
//! Runs the full Extract → Transform pipeline against SEC EDGAR to measure
//! pipeline quality and track progress as XBRL aliases are expanded.
//!
//! ## Running
//! ```sh
//! # Run both tests sequentially with live output
//! cargo test --test pipeline_coverage -- --ignored --nocapture --test-threads=1
//! ```

mod builder;
mod constants;

use std::collections::HashMap;
use std::io::Write;

use futures_util::StreamExt;

use builder::Pipeline;
use constants::{MUST_PASS_CIKS, SP500_CIKS};

/// Writes directly to stderr, bypassing the test framework's output capture.
fn write_progress(msg: &str) {
    let _ = std::io::stderr().write_all(msg.as_bytes());
    let _ = std::io::stderr().flush();
}

/// Looks up the company name for a CIK in the must-pass list.
///
/// Returns "Unknown" if the CIK is not in the list.
fn lookup_must_pass_name(cik: &str) -> &str {
    MUST_PASS_CIKS
        .iter()
        .find(|(c, _)| *c == cik)
        .map_or("Unknown", |(_, name)| name)
}

/// Minimum number of S&P 500 companies that must succeed.
/// Increase this threshold as XBRL aliases are expanded.
const MINIMUM_SUCCESS_THRESHOLD: usize = 270;

// ---------------------------------------------------------------------------
// Clean end-to-end test — mirrors the production binary
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
#[ignore = "Hits the live SEC API for must-pass companies (~10 seconds)"]
async fn should_succeed_for_must_pass_companies() {
    let ciks: Vec<&str> = MUST_PASS_CIKS.iter().map(|(cik, _)| *cik).collect();
    let stream_results: Vec<_> = futures_util::stream::iter(ciks.iter())
        .map(|cik| Pipeline::builder().cik(*cik).build().run())
        .buffer_unordered(10)
        .collect()
        .await;

    let expected_result: Vec<String> = vec![];

    let result: Vec<String> = stream_results
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            r.as_ref().err().map(|e| {
                let name = lookup_must_pass_name(ciks[i]);
                format!("{name} (CIK {}): {e}", ciks[i])
            })
        })
        .collect();

    assert_eq!(result, expected_result);
}

// ---------------------------------------------------------------------------
// Diagnostic test — live progress, per-CIK errors, aggregated breakdown
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
#[ignore = "Hits the live SEC API for all S&P 500 CIKs (~3 minutes)"]
async fn should_meet_threshold_for_sp500_companies() {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let ciks: Vec<&str> = SP500_CIKS.to_vec();
    let total = ciks.len();
    let completed = AtomicUsize::new(0);
    let successes = AtomicUsize::new(0);
    let failures: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

    let expected_result = true;

    futures_util::stream::iter(ciks.iter())
        .for_each_concurrent(10, |cik| {
            let completed = &completed;
            let successes = &successes;
            let failures = &failures;
            async move {
                let result = Pipeline::builder().cik(*cik).build().run().await;
                let n = completed.fetch_add(1, Ordering::Relaxed) + 1;
                match result {
                    Ok(()) => {
                        successes.fetch_add(1, Ordering::Relaxed);
                        write_progress(&format!("[{n:>4}/{total}] CIK {cik} ✓\n"));
                    }
                    Err(e) => {
                        write_progress(&format!("[{n:>4}/{total}] CIK {cik} ✗\n"));
                        failures
                            .lock()
                            .expect("Mutex should not be poisoned")
                            .push((cik.to_string(), e.to_string()));
                    }
                }
            }
        })
        .await;

    let success_count = successes.load(Ordering::Relaxed);
    let failure_list = failures.into_inner().expect("Mutex should not be poisoned");

    write_progress(&format!(
        "\n{}\nPipeline Coverage: {success_count}/{total} passed ({} failed)\n{}\n",
        "=".repeat(60),
        failure_list.len(),
        "=".repeat(60),
    ));

    if !failure_list.is_empty() {
        write_progress("\nFailed CIKs:\n");
        for (cik, error_chain) in &failure_list {
            write_progress(&format!("\n  CIK {cik}:\n    {error_chain}\n"));
        }

        let mut field_counts: HashMap<String, usize> = HashMap::new();
        for (_, error_chain) in &failure_list {
            if let Some(start) = error_chain.find("Missing fields: [\"") {
                let fields_start = start + "Missing fields: [\"".len();
                if let Some(end) = error_chain[fields_start..].find("\"]") {
                    let fields_str = &error_chain[fields_start..fields_start + end];
                    for field in fields_str.split("\", \"") {
                        *field_counts.entry(field.to_string()).or_default() += 1;
                    }
                }
            }
        }
        if !field_counts.is_empty() {
            let mut sorted: Vec<_> = field_counts.into_iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(&a.1));
            write_progress("\nMissing Field Breakdown:\n");
            for (field, count) in &sorted {
                write_progress(&format!("  {count:>4}x {field}\n"));
            }
        }
    }

    write_progress(&format!(
        "\nThreshold: {success_count} >= {MINIMUM_SUCCESS_THRESHOLD}\n"
    ));

    let result = success_count >= MINIMUM_SUCCESS_THRESHOLD;

    assert_eq!(result, expected_result);
}
