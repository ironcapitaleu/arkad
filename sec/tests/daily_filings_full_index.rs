//! # `daily_filings` from the Full Index Tests
//!
//! Runs `run_from_full_index` of the `Extraction` facade offline, with a fake SEC client and stores
//! in a temp dir. The catch-up of `backfill` calls it.
//!
//! The fake SEC serves the cut full indexes of 2026 Q3 and Q4. Every filing folder answers 404, so
//! each selected filing fails and the summary names it.

use std::time::Duration;

use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use sec::implementations::extraction::{DailyFilingsRepositories, DailyRunSummary, Extraction};
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

/// Runs `dates` from the full index with the stores under `root`, `as_of`, and no wait between
/// attempts.
async fn run(
    root: &TempDir,
    dates: &[NaiveDate],
    as_of: NaiveDate,
    sec_client: &RoutingSecClient,
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
            |_, _| {},
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

#[tokio::test]
async fn should_count_the_dates_of_a_recent_quarter_as_pending_when_its_full_index_is_absent() {
    let dates = [date("2026-09-30"), date("2026-10-01")];
    let sec_client = sec_client(404);

    let expected_result = vec!["processed", "pending"];

    let result: Vec<String> = run(&temporary_root(), &dates, date("2026-10-02"), &sec_client)
        .await
        .entries()
        .iter()
        .map(|entry| entry["detection"].as_str().unwrap_or_default().to_string())
        .collect();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_fail_each_date_of_a_quarter_when_its_full_index_is_absent_on_the_first_date() {
    let dates = [date("2026-10-01"), date("2026-10-02")];
    let sec_client = sec_client(404);

    let expected_result = vec!["failed", "failed"];

    let result: Vec<String> = run(&temporary_root(), &dates, date("2026-10-02"), &sec_client)
        .await
        .entries()
        .iter()
        .map(|entry| entry["detection"].as_str().unwrap_or_default().to_string())
        .collect();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_fail_each_date_of_a_quarter_when_its_full_index_answers_500_on_every_attempt() {
    let dates = [date("2026-09-30"), date("2026-10-01")];
    let sec_client = sec_client(500);

    let expected_result = vec!["processed", "failed"];

    let result: Vec<String> = run(&temporary_root(), &dates, date("2026-10-02"), &sec_client)
        .await
        .entries()
        .iter()
        .map(|entry| entry["detection"].as_str().unwrap_or_default().to_string())
        .collect();

    assert_eq!(result, expected_result);
}
