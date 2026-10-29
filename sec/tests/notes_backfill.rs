//! # `notes_backfill` Integration Tests
//!
//! Runs the backfill loop of the `notes_backfill` binary over the two smallest Notes files, and
//! checks the run summary.
//!
//! ## External Dependencies
//!
//! - `www.sec.gov`: the Notes files of 2009q1 (28 KB) and 2009q2 (534 KB)

// The binary is its own crate root, so this test compiles its loop as a private module.
#[path = "../src/bin/notes_backfill/backfill.rs"]
mod backfill;

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use storage::implementations::repository::filesystem::FilesystemRepository;
use tempfile::TempDir;

use sec::implementations::extraction::Extraction;
use sec::shared::http_client::implementations::reqwest_client::ReqwestClient;
use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::RawNotesArchive;
use sec::shared::user_agent::UserAgent;

use backfill::{FailedBackfill, RunSummary};

/// A month far after the last published file, so the SEC answers 404.
const UNPUBLISHED_PERIOD: &str = "2099_01";

fn sec_client() -> SecClient {
    let user_agent = UserAgent::new("ArkadTest admin@example.com")
        .expect("A hardcoded SEC-format string should always validate");
    let http_client = ReqwestClient::new(user_agent)
        .expect("Given a validated user agent, building the HTTP client fails only if the TLS backend cannot initialize");
    SecClient::new(http_client)
}

/// Runs the backfill loop over `periods` into a new temporary root.
async fn run_backfill(periods: &[&str]) -> RunSummary {
    let root = TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    );
    let repository = FilesystemRepository::builder()
        .writer()
        .document_type::<RawDocument<RawNotesArchive>>()
        .root(root.path())
        .build();
    backfill::run(
        Extraction::backfill(),
        periods,
        &sec_client(),
        &repository,
        |_, _| {},
        |_, _| {},
    )
    .await
}

#[tokio::test]
async fn should_list_each_stored_file_with_its_size_and_sha256_when_backfilling_published_periods()
{
    let expected_result = vec![
        json!({
            "period": "2009q1",
            "key": "sec/notes/2009q1_notes.zip",
            "size_in_bytes": 28_064,
            "sha256": "ebdbbe58d6908d2c9e2dcb5c547be1617040c3ee7dc9a287b2e49f197abd4b4f",
        }),
        json!({
            "period": "2009q2",
            "key": "sec/notes/2009q2_notes.zip",
            "size_in_bytes": 533_925,
            "sha256": "3ca835467d2eec762ca064b975ebf4d34add69bdd04d13fbd5bb82b304e852b6",
        }),
    ];

    let result = run_backfill(&["2009q1", "2009q2"]).await.entries();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_store_the_other_periods_when_one_period_fails() {
    let expected_result = vec![
        json!("sec/notes/2009q1_notes.zip"),
        json!("Download Notes Archive"),
        json!("sec/notes/2009q2_notes.zip"),
    ];

    let result: Vec<Value> = run_backfill(&["2009q1", UNPUBLISHED_PERIOD, "2009q2"])
        .await
        .entries()
        .into_iter()
        .map(|entry| {
            entry
                .get("key")
                .unwrap_or_else(|| &entry["failed_state"])
                .clone()
        })
        .collect();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_failed_backfill_when_one_period_is_unpublished() {
    let expected_result = Err(FailedBackfill {
        failures: 1,
        periods: 3,
    });

    let result = run_backfill(&["2009q1", UNPUBLISHED_PERIOD, "2009q2"])
        .await
        .result();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_ok_when_every_period_is_stored() {
    let expected_result = Ok(());

    let result = run_backfill(&["2009q1"]).await.result();

    assert_eq!(result, expected_result);
}
