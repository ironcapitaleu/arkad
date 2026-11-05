//! # Notes Retry Integration Tests
//!
//! Runs the Notes batch run of the `Extraction` facade offline, with a fake SEC client whose first
//! body stops early, and checks that the run downloads the file again.
//!
//! ## External Dependencies
//!
//! None. The fake SEC client builds every response locally.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use pretty_assertions::assert_eq;
use storage::implementations::repository::filesystem::FilesystemRepository;
use tempfile::TempDir;

use sec::implementations::extraction::{Extraction, NotesRunSummary, RunEvent};
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::RawNotesArchive;
use sec::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
use sec::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;

const PERIOD: &str = "2026_09";
const NOTES_URL: &str =
    "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2026_09_notes.zip";
const LOCK_EXPECT: &str = "Given an observer that never panics while it holds the lock, the lock should never be poisoned";

/// Returns a client whose first body for the Notes file of [`PERIOD`] stops early, and whose
/// later answers send the whole file.
fn sec_client_with_a_stopped_body() -> RoutingSecClient {
    RoutingSecClient::new().route_stopped_body_once(NOTES_URL, "application/zip", SAMPLE_NOTES_ZIP)
}

/// Runs the Notes batch over [`PERIOD`] into a new temporary root, with no wait between
/// attempts, and passes each event to `observer`.
async fn run_observed(
    sec_client: RoutingSecClient,
    observer: impl Fn(RunEvent<'_>) + Send + Sync + 'static,
) -> NotesRunSummary {
    let root = TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    );
    let repository = FilesystemRepository::builder()
        .writer()
        .document_type::<RawDocument<RawNotesArchive>>()
        .root(root.path())
        .build();
    Extraction::backfill()
        .notes()
        .periods([PERIOD])
        .sec_client(sec_client)
        .repository(repository)
        .retry_delays([Duration::ZERO, Duration::ZERO])
        .run(move |_, event| observer(event))
        .await
}

#[tokio::test]
async fn should_store_the_file_when_the_first_body_stops_early() {
    let sec_client = sec_client_with_a_stopped_body();

    let expected_result = 1;

    let result = run_observed(sec_client, |_| {}).await.success_count();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_report_one_retried_error_when_the_first_body_stops_early() {
    let sec_client = sec_client_with_a_stopped_body();
    let kinds = Arc::new(Mutex::new(Vec::new()));
    let observed_kinds = Arc::clone(&kinds);

    let expected_result = vec!["retried".to_string()];

    let _ = run_observed(sec_client, move |event| {
        if let RunEvent::Error(_, kind) = event {
            observed_kinds
                .lock()
                .expect(LOCK_EXPECT)
                .push(kind.to_string());
        }
    })
    .await;
    let result = kinds.lock().expect(LOCK_EXPECT).clone();

    assert_eq!(result, expected_result);
}
