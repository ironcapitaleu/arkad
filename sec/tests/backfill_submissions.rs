//! # `backfill().submissions()` Example Tests
//!
//! Shows how to backfill the `submissions` file of one company through the `Extraction` facade,
//! for example to reconcile that company by hand. No binary runs this call.
//!
//! ```text
//! let state = Extraction::backfill()
//!     .submissions()
//!     .cik("1067983")
//!     .sec_client(sec_client)
//!     .repository(repository)
//!     .build();
//! let mut stream = state.into_run_stream();
//! while let Some(item) = stream.next().await {
//!     item?;
//! }
//! ```
//!
//! ## External Dependencies
//!
//! None. A fake SEC client answers every request, and the store writes to a temp dir.

use futures_util::StreamExt;
use pretty_assertions::assert_eq;
use sec::implementations::extraction::Extraction;
use sec::prelude::StreamError;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::RawSubmissions;
use sec::tests::fixtures::data::submissions::SAMPLE_SUBMISSIONS_JSON;
use sec::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
use serde_json::Value;
use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
use tempfile::TempDir;
use uuid::Uuid;

type SubmissionsRepository = FilesystemRepository<Write, RawDocument<RawSubmissions>>;

/// The CIK of Berkshire Hathaway, the company of [`SAMPLE_SUBMISSIONS_JSON`].
const CIK: &str = "1067983";
const SUBMISSIONS_URL: &str = "https://data.sec.gov/submissions/CIK0001067983.json";
const STORED_FILE: &str = "sec/submissions/CIK0001067983.json";
/// A later `submissions` body of the same company, with one more filing.
const NEWER_SUBMISSIONS_JSON: &[u8] = br#"{"cik":"1067983","name":"BERKSHIRE HATHAWAY INC","filings":{"recent":{"accessionNumber":["0000950170-26-099999","0000950170-26-012345"],"filingDate":["2026-11-02","2026-08-04"],"form":["10-Q","10-Q"]}}}"#;

fn temporary_root() -> TempDir {
    TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    )
}

fn repository(root: &TempDir) -> SubmissionsRepository {
    FilesystemRepository::builder()
        .writer()
        .document_type::<RawDocument<RawSubmissions>>()
        .root(root.path())
        .build()
}

/// Returns a client that answers the `submissions` URL of [`CIK`] with `body`.
fn sec_client(body: &[u8]) -> RoutingSecClient {
    RoutingSecClient::new().route(SUBMISSIONS_URL, 200, "application/json", body)
}

/// Backfills the `submissions` file of [`CIK`] into `root`, with the SEC answering `body`.
async fn backfill_one_company(root: &TempDir, body: &[u8]) -> Result<(), StreamError> {
    let state = Extraction::backfill()
        .execution_id(Uuid::from_u128(7))
        .submissions()
        .cik(CIK)
        .sec_client(sec_client(body))
        .repository(repository(root))
        .build();
    let mut stream = state.into_run_stream();
    while let Some(item) = stream.next().await {
        item?;
    }
    Ok(())
}

fn stored_file(root: &TempDir) -> Vec<u8> {
    std::fs::read(root.path().join(STORED_FILE))
        .expect("Given a run that stored the document, the file should always exist")
}

fn stored_metadata(root: &TempDir) -> Value {
    serde_json::from_slice(
        &std::fs::read(root.path().join(format!("{STORED_FILE}.meta.json")))
            .expect("Given a run that stored the document, its metadata file should always exist"),
    )
    .expect("Given a metadata file that the store wrote, it should always parse as JSON")
}

#[tokio::test]
async fn should_store_the_submissions_file_of_the_company_when_backfilling_one_cik() {
    let root = temporary_root();

    let expected_result = SAMPLE_SUBMISSIONS_JSON.to_vec();

    backfill_one_company(&root, SAMPLE_SUBMISSIONS_JSON)
        .await
        .expect("Given an SEC client that answers the CIK, the machine should always succeed");
    let result = stored_file(&root);

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_record_the_backfill_mode_and_the_execution_id_when_backfilling_one_cik() {
    let root = temporary_root();

    let expected_result = (
        Value::from("backfill"),
        Value::from("00000000-0000-0000-0000-000000000007"),
    );

    backfill_one_company(&root, SAMPLE_SUBMISSIONS_JSON)
        .await
        .expect("Given an SEC client that answers the CIK, the machine should always succeed");
    let metadata = stored_metadata(&root);
    let result = (
        metadata["ingestion_mode"].clone(),
        metadata["execution_id"].clone(),
    );

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_replace_the_stored_file_when_reconciling_a_company_whose_sec_body_changed() {
    let root = temporary_root();
    backfill_one_company(&root, SAMPLE_SUBMISSIONS_JSON)
        .await
        .expect("Given an SEC client that answers the CIK, the first run should always succeed");

    let expected_result = NEWER_SUBMISSIONS_JSON.to_vec();

    backfill_one_company(&root, NEWER_SUBMISSIONS_JSON)
        .await
        .expect("Given an SEC client that answers the CIK, the second run should always succeed");
    let result = stored_file(&root);

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_fail_the_machine_when_the_sec_has_no_submissions_file_of_the_cik() {
    let root = temporary_root();
    let state = Extraction::backfill()
        .submissions()
        .cik(CIK)
        .sec_client(RoutingSecClient::new())
        .repository(repository(&root))
        .build();

    let expected_result = true;

    let mut stream = state.into_run_stream();
    let mut failed = false;
    while let Some(item) = stream.next().await {
        failed |= item.is_err();
    }
    let result = failed;

    assert_eq!(result, expected_result);
}
