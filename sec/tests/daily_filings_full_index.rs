//! # `daily_filings` from the Full Index Tests
//!
//! Runs `run_from_full_index` of the `Extraction` facade offline, with a fake SEC client and stores
//! in a temp dir. The catch-up of `backfill` calls it.
//!
//! The fake SEC serves the cut full indexes of 2026 Q3 and Q4. Every filing folder answers 404, so
//! each selected filing fails and the summary names it.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use sec::implementations::extraction::{
    DailyFilingsRepositories, DailyRunSummary, Extraction, RunEvent, Unit,
};
use sec::shared::raw_document::RawDocument;
use sec::tests::fixtures::data::full_index::{
    SAMPLE_FULL_INDEX_2026_Q3, SAMPLE_FULL_INDEX_2026_Q4,
};
use sec::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
use serde_json::{Value, json};
use storage::implementations::repository::filesystem::FilesystemRepository;
use tempfile::TempDir;
use uuid::Uuid;

const FULL_INDEX_URL_OF_2026_Q3: &str =
    "https://www.sec.gov/Archives/edgar/full-index/2026/QTR3/form.idx";
const FULL_INDEX_URL_OF_2026_Q4: &str =
    "https://www.sec.gov/Archives/edgar/full-index/2026/QTR4/form.idx";

fn date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .expect("Given a hardcoded valid date, parsing should always succeed")
}

fn temporary_root() -> TempDir {
    TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    )
}

/// Returns a client that serves the full index of 2026 Q3, and answers `q4_status` for the full
/// index of 2026 Q4. Every other URL answers 404.
fn sec_client(q4_status: u16) -> RoutingSecClient {
    RoutingSecClient::new()
        .route(
            FULL_INDEX_URL_OF_2026_Q3,
            200,
            "application/octet-stream",
            SAMPLE_FULL_INDEX_2026_Q3.as_bytes(),
        )
        .route(
            FULL_INDEX_URL_OF_2026_Q4,
            q4_status,
            "application/octet-stream",
            SAMPLE_FULL_INDEX_2026_Q4.as_bytes(),
        )
}

fn repositories(root: &TempDir) -> DailyFilingsRepositories {
    DailyFilingsRepositories {
        daily_index: FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<_>>()
            .root(root.path())
            .build(),
        filing_index: FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<_>>()
            .root(root.path())
            .build(),
        instance: FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<_>>()
            .root(root.path())
            .build(),
        labels: FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<_>>()
            .root(root.path())
            .build(),
        submissions: FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<_>>()
            .root(root.path())
            .build(),
    }
}

const LOCK_EXPECT: &str = "Given an observer that never panics while it holds the lock, the lock should never be poisoned";

/// Runs `dates` as `run_observed` does, without an observer.
async fn run(
    root: &TempDir,
    dates: &[NaiveDate],
    as_of: NaiveDate,
    sec_client: &RoutingSecClient,
) -> DailyRunSummary {
    run_observed(root, dates, as_of, sec_client, |_, _| {}).await
}

/// Runs `dates` from the full index with the stores under `root`, `as_of`, one retry with no wait,
/// and passes each event to `observer`.
async fn run_observed(
    root: &TempDir,
    dates: &[NaiveDate],
    as_of: NaiveDate,
    sec_client: &RoutingSecClient,
    observer: impl Fn(&Unit, RunEvent<'_>) + Send + Sync + 'static,
) -> DailyRunSummary {
    Extraction::incremental()
        .execution_id(Uuid::from_u128(7))
        .daily_filings()
        .dates(dates.iter().copied())
        .as_of(as_of)
        .sec_client(sec_client.clone())
        .repositories(repositories(root))
        .retry_delays([Duration::ZERO])
        .run_from_full_index(
            FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<_>>()
                .root(root.path())
                .build(),
            observer,
        )
        .await
}

/// Returns the date, the detection, and the accession number of each failed filing of each entry.
fn failed_filings(summary: &DailyRunSummary) -> Vec<Value> {
    summary
        .entries()
        .iter()
        .map(|entry| {
            let accession_numbers: Vec<&Value> = entry["filings"]["failed"]
                .as_array()
                .map(|failed| {
                    failed
                        .iter()
                        .map(|filing| &filing["accession_number"])
                        .collect()
                })
                .unwrap_or_default();
            json!({
                "date": entry["date"],
                "detection": entry["detection"],
                "failed": accession_numbers,
            })
        })
        .collect()
}

#[tokio::test]
async fn should_select_the_filings_of_the_daily_index_of_each_date_when_the_dates_span_two_quarters()
 {
    let dates = [date("2026-09-30"), date("2026-10-01")];
    let sec_client = sec_client(200);

    // The daily indexes of 2026-09-30 and 2026-10-01 list the same selected rows, in this order.
    let expected_result = vec![
        json!({
            "date": "2026-09-30",
            "detection": "processed",
            "failed": [
                "0001493152-26-044953",
                "0001193125-26-409138",
                "0001170010-26-000104",
                "0001104659-26-112415",
            ],
        }),
        json!({
            "date": "2026-10-01",
            "detection": "processed",
            "failed": [
                "0001437749-26-031727",
                "0000063754-26-000329",
                "0001493152-26-045181",
            ],
        }),
    ];

    let result =
        failed_filings(&run(&temporary_root(), &dates, date("2026-10-31"), &sec_client).await);

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_count_a_date_as_empty_when_the_full_index_lists_no_filing_with_that_date() {
    let dates = [date("2026-10-03")];
    let sec_client = sec_client(200);

    let expected_result = vec![json!({
        "date": "2026-10-03",
        "detection": "empty",
        "failed": [],
    })];

    let result =
        failed_filings(&run(&temporary_root(), &dates, date("2026-10-31"), &sec_client).await);

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_store_the_full_index_of_each_quarter_when_the_dates_span_two_quarters() {
    let root = temporary_root();
    let dates = [date("2026-09-30"), date("2026-10-01")];
    let sec_client = sec_client(200);

    let expected_result = (true, true);

    let _ = run(&root, &dates, date("2026-10-31"), &sec_client).await;
    let result = (
        root.path()
            .join("sec/full-index/2026/QTR3/form.idx")
            .exists(),
        root.path()
            .join("sec/full-index/2026/QTR4/form.idx")
            .exists(),
    );

    assert_eq!(result, expected_result);
}

/// Runs `dates` as `run` does in a new temp dir, and returns the detection of each date.
async fn detections(
    dates: &[NaiveDate],
    as_of: NaiveDate,
    sec_client: &RoutingSecClient,
) -> Vec<String> {
    run(&temporary_root(), dates, as_of, sec_client)
        .await
        .entries()
        .iter()
        .map(|entry| entry["detection"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[tokio::test]
async fn should_count_each_date_of_a_recent_quarter_as_pending_when_its_full_index_is_absent() {
    let dates = [date("2026-09-30"), date("2026-10-01"), date("2026-10-02")];
    let sec_client = sec_client(404);

    let expected_result = vec!["processed", "pending", "pending"];

    let result = detections(&dates, date("2026-10-03"), &sec_client).await;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_fail_each_date_of_a_quarter_when_its_full_index_is_absent_on_the_first_date() {
    let dates = [date("2026-10-01"), date("2026-10-02")];
    let sec_client = sec_client(404);

    let expected_result = vec!["failed", "failed"];

    let result = detections(&dates, date("2026-10-02"), &sec_client).await;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_fail_each_date_of_a_quarter_when_its_full_index_answers_500() {
    let dates = [date("2026-09-30"), date("2026-10-01"), date("2026-10-02")];
    let sec_client = sec_client(500);

    let expected_result = vec!["processed", "failed", "failed"];

    let result = detections(&dates, date("2026-10-03"), &sec_client).await;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_fail_each_date_of_a_quarter_when_its_full_index_answers_503_on_every_attempt() {
    let dates = [date("2026-09-30"), date("2026-10-01"), date("2026-10-02")];
    let sec_client = sec_client(503);

    let expected_result = vec!["processed", "failed", "failed"];

    let result = detections(&dates, date("2026-10-03"), &sec_client).await;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_report_a_retried_and_a_failed_error_with_the_quarter_when_its_full_index_answers_503()
 {
    let dates = [date("2026-09-30"), date("2026-10-01")];
    let sec_client = sec_client(503);
    let errors = Arc::new(Mutex::new(Vec::new()));
    let observed_errors = Arc::clone(&errors);
    let observer = move |unit: &Unit, event: RunEvent<'_>| {
        if let (Unit::Quarter(_), RunEvent::Error(_, kind)) = (unit, event) {
            observed_errors
                .lock()
                .expect(LOCK_EXPECT)
                .push(json!({ "context": unit, "kind": kind.to_string() }));
        }
    };

    let expected_result = vec![
        json!({ "context": { "quarter": "2026/QTR4" }, "kind": "retried" }),
        json!({ "context": { "quarter": "2026/QTR4" }, "kind": "failed" }),
    ];

    let _ = run_observed(
        &temporary_root(),
        &dates,
        date("2026-10-31"),
        &sec_client,
        observer,
    )
    .await;
    let result = errors.lock().expect(LOCK_EXPECT).clone();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_count_a_recent_date_as_pending_when_it_comes_after_the_last_data_received() {
    // The full index of Q4 names 2026-11-12 as its last data received.
    let dates = [date("2026-11-12"), date("2026-11-13")];
    let sec_client = sec_client(200);

    let expected_result = vec!["empty", "pending"];

    let result = detections(&dates, date("2026-11-14"), &sec_client).await;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_fail_an_old_date_when_it_comes_after_the_last_data_received() {
    let dates = [date("2026-11-12"), date("2026-11-13")];
    let sec_client = sec_client(200);

    let expected_result = vec!["empty", "failed"];

    let result = detections(&dates, date("2026-11-30"), &sec_client).await;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_fail_the_first_date_when_it_comes_after_the_last_data_received() {
    let dates = [date("2026-11-13")];
    let sec_client = sec_client(200);

    let expected_result = vec!["failed"];

    let result = detections(&dates, date("2026-11-14"), &sec_client).await;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_name_the_date_and_the_last_data_received_when_a_date_is_pending() {
    let dates = [date("2026-11-12"), date("2026-11-13")];
    let sec_client = sec_client(200);

    let expected_result = json!(
        "[StreamError][00000000-0000-0000-0000-000000000007] 'state_failed' in 'Select Full Index Filings', Caused by: [MissingFullIndexDate] The full index does not cover the date yet, Date: '2026-11-13', Last Data Received: '2026-11-12'"
    );

    let result = run(&temporary_root(), &dates, date("2026-11-14"), &sec_client)
        .await
        .entries()[1]["error"]
        .clone();

    assert_eq!(result, expected_result);
}
