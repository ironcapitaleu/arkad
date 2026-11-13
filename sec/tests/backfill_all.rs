//! # `backfill().all()` Tests
//!
//! Runs the pipeline of a backfill run offline: the cut-over probe, then the Notes step and the
//! catch-up at the same time. The `backfill` binary makes this call.
//!
//! ```text
//! let summary = Extraction::backfill()
//!     .all()
//!     .periods(periods)
//!     .until(last_date)
//!     .sec_client(sec_client)
//!     .repositories(repositories)
//!     .run(observer)
//!     .await?;
//! ```
//!
//! ## External Dependencies
//!
//! None. A fake SEC client answers every request, and the stores write to a temp dir.

use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use sec::implementations::extraction::{
    AllRunSummary, BackfillRepositories, DailyFilingsRepositories, Extraction, FailedAllBackfill,
};
use sec::shared::raw_document::RawDocument;
use sec::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
use sec::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
use serde_json::{Value, json};
use storage::implementations::repository::filesystem::FilesystemRepository;
use tempfile::TempDir;
use uuid::Uuid;

const NOTES_URL: &str = "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/";
const INDEX_URL_OF_2026_09_30: &str =
    "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR3/form.20260930.idx";
const INDEX_URL_OF_2026_10_01: &str =
    "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261001.idx";
/// A daily index that lists no filing.
const EMPTY_DAILY_INDEX: &[u8] = b"Form Type   Company Name   CIK   Date Filed  File Name
---------------------------------------------------------------------
";
const RUN_EXPECT: &str =
    "Given a published Notes period in the list, the run should always find a cut-over date";

fn date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .expect("Given a hardcoded valid date, parsing should always succeed")
}

fn temporary_root() -> TempDir {
    TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    )
}

/// Returns a client that serves the Notes file of each of `periods` and an empty daily index of
/// 2026-09-30 and of 2026-10-01. Every other URL answers 404.
fn sec_client(periods: &[&str]) -> RoutingSecClient {
    periods
        .iter()
        .fold(RoutingSecClient::new(), |sec_client, period| {
            sec_client.route(
                &format!("{NOTES_URL}{period}_notes.zip"),
                200,
                "application/zip",
                SAMPLE_NOTES_ZIP,
            )
        })
        .route(
            INDEX_URL_OF_2026_09_30,
            200,
            "application/octet-stream",
            EMPTY_DAILY_INDEX,
        )
        .route(
            INDEX_URL_OF_2026_10_01,
            200,
            "application/octet-stream",
            EMPTY_DAILY_INDEX,
        )
}

fn repositories(root: &TempDir) -> BackfillRepositories {
    BackfillRepositories {
        notes: FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<_>>()
            .root(root.path())
            .build(),
        daily_filings: DailyFilingsRepositories {
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
        },
    }
}

/// Runs the pipeline over `periods` into `root` with the `execution_id` 7, until 2026-10-01, and
/// without an observer.
async fn run_all(root: &TempDir, sec_client: RoutingSecClient, periods: &[&str]) -> AllRunSummary {
    Extraction::backfill()
        .execution_id(Uuid::from_u128(7))
        .all()
        .periods(periods.iter().copied())
        .until(date("2026-10-01"))
        .sec_client(sec_client)
        .repositories(repositories(root))
        .run(|_, _| {})
        .await
        .expect(RUN_EXPECT)
}

/// Returns the date of each entry of the catch-up of `summary`.
fn catch_up_dates(summary: &AllRunSummary) -> Value {
    summary
        .catch_up()
        .entries()
        .iter()
        .map(|entry| entry["date"].clone())
        .collect()
}

/// Reads the metadata file at `file` under `root` as JSON.
fn stored_metadata(root: &TempDir, file: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(root.path().join(file))
            .expect("Given a run that stored the document, its metadata file should always exist"),
    )
    .expect("Given a metadata file that the store wrote, it should always parse as JSON")
}

#[tokio::test]
async fn should_run_the_catch_up_from_the_cut_over_date_to_until_when_the_newest_period_is_published()
 {
    let root = temporary_root();
    let sec_client = sec_client(&["2026_09"]);
    let periods = ["2026_09"];

    let expected_result = json!(["2026-09-30", "2026-10-01"]);

    let summary = run_all(&root, sec_client, &periods).await;
    let result = catch_up_dates(&summary);

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_record_one_execution_id_for_the_notes_file_and_the_daily_index_when_both_steps_run()
{
    let root = temporary_root();
    let sec_client = sec_client(&["2026_09"]);
    let periods = ["2026_09"];
    let metadata_files = [
        "sec/notes/2026_09_notes.zip.meta.json",
        "sec/daily-index/form.20260930.idx.meta.json",
    ];

    let expected_result = json!([
        "00000000-0000-0000-0000-000000000007",
        "00000000-0000-0000-0000-000000000007",
    ]);

    run_all(&root, sec_client, &periods).await;
    let result: Value = metadata_files
        .iter()
        .map(|file| stored_metadata(&root, file)["execution_id"].clone())
        .collect();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_start_the_catch_up_one_month_back_and_succeed_when_the_newest_period_is_unpublished()
 {
    let root = temporary_root();
    let sec_client = sec_client(&["2026_09"]);
    let periods = ["2026_09", "2026_10"];

    let expected_result = (date("2026-09-30"), Ok(()));

    let summary = run_all(&root, sec_client, &periods).await;
    let result = (summary.cut_over_date(), summary.result());

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_run_the_catch_up_when_a_notes_period_fails() {
    let root = temporary_root();
    let sec_client = sec_client(&["2026_09"]);
    let periods = ["2026_08", "2026_09"];

    let expected_result = (1, json!(["2026-09-30", "2026-10-01"]));

    let summary = run_all(&root, sec_client, &periods).await;
    let result = (summary.notes().failure_count(), catch_up_dates(&summary));

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_fail_the_older_period_when_the_list_is_not_in_order() {
    let root = temporary_root();
    let sec_client = sec_client(&["2026_09"]);
    let periods = ["2026_09", "2026_08"];

    let expected_result = Err(FailedAllBackfill {
        failed_periods: 1,
        periods: 2,
        catch_up_failures: 0,
        catch_up_dates: 2,
    });

    let result = run_all(&root, sec_client, &periods).await.result();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_run_no_catch_up_date_when_until_is_before_the_cut_over_date() {
    let root = temporary_root();
    let sec_client = sec_client(&["2026_09"]);
    let periods = ["2026_09"];
    let until = date("2026-09-01");

    let expected_result = (json!([]), Ok(()));

    let summary = Extraction::backfill()
        .all()
        .periods(periods)
        .until(until)
        .sec_client(sec_client)
        .repositories(repositories(&root))
        .run(|_, _| {})
        .await
        .expect(RUN_EXPECT);
    let result = (catch_up_dates(&summary), summary.result());

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_failed_all_backfill_when_a_notes_period_fails() {
    let root = temporary_root();
    let sec_client = sec_client(&["2026_09"]);
    let periods = ["2026_08", "2026_09"];

    let expected_result = Err(FailedAllBackfill {
        failed_periods: 1,
        periods: 2,
        catch_up_failures: 0,
        catch_up_dates: 2,
    });

    let result = run_all(&root, sec_client, &periods).await.result();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_download_the_failed_period_when_a_second_run_repeats_the_run() {
    let root = temporary_root();
    let periods = ["2026_08", "2026_09"];
    run_all(&root, sec_client(&["2026_09"]), &periods).await;

    let expected_result = json!([
        { "period": "2026_08", "status": "downloaded" },
        { "period": "2026_09", "status": "already_stored" },
    ]);

    let summary = run_all(&root, sec_client(&["2026_08", "2026_09"]), &periods).await;
    let result: Value = summary
        .notes()
        .entries()
        .iter()
        .map(|entry| json!({ "period": entry["period"], "status": entry["status"] }))
        .collect();

    assert_eq!(result, expected_result);
}
