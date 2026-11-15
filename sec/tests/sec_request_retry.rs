//! # Request Retry Integration Tests
//!
//! Sends a Notes request through a fake SEC client with a [`RetryPolicy`] that answers 503, and
//! checks that the request succeeds on a later attempt or fails after the maximum. One test runs
//! the Notes batch of the `Extraction` facade with its run-level retry off.
//!
//! ## External Dependencies
//!
//! None. The fake SEC client builds every response locally.

use std::collections::BTreeMap;
use std::time::Duration;

use pretty_assertions::assert_eq;
use storage::implementations::repository::filesystem::FilesystemRepository;
use tempfile::TempDir;

use sec::implementations::extraction::Extraction;
use sec::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use sec::shared::http_client::{RetryPolicy, SecClient};
use sec::shared::notes_period::NotesPeriod;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::RawNotesArchive;
use sec::shared::request::implementations::sec_request::{NotesFile, SecRequest};
use sec::shared::response::implementations::sec_response::error::{
    ErrorReason, InvalidSecResponse,
};
use sec::shared::status_code::StatusCode;
use sec::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
use sec::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;

const PERIOD: &str = "2026_09";
const NOTES_URL: &str =
    "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2026_09_notes.zip";
const ZIP: &str = "application/zip";

/// The default number of attempts, without waits.
const NO_WAIT_POLICY: RetryPolicy = RetryPolicy {
    max_attempts: 3,
    first_wait: Duration::ZERO,
    factor: 2,
    max_wait: Duration::ZERO,
};

/// Returns a fake SEC client that retries, and answers the Notes file of [`PERIOD`] with
/// `responses` in order.
fn retrying_client(responses: &[(u16, &str, &[u8])]) -> RoutingSecClient {
    RoutingSecClient::new()
        .route_sequence(NOTES_URL, responses)
        .with_retry_policy(NO_WAIT_POLICY)
}

/// Returns a fake SEC client that retries, and answers 503 to every request.
fn always_unavailable_client() -> RoutingSecClient {
    retrying_client(&[(503, "text/html", b"")])
}

fn notes_request() -> SecRequest<NotesFile> {
    let period = NotesPeriod::new(PERIOD)
        .expect("Given a hardcoded published period, parsing should always succeed");
    SecRequest::builder().notes_file().period(period).build()
}

#[tokio::test]
async fn should_store_the_file_when_the_sec_answers_503_once_then_200() {
    let sec_client = retrying_client(&[(503, "text/html", b""), (200, ZIP, SAMPLE_NOTES_ZIP)]);
    let root = TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    );
    let repository = FilesystemRepository::builder()
        .writer()
        .document_type::<RawDocument<RawNotesArchive>>()
        .root(root.path())
        .build();

    let expected_result = 1;

    let result = Extraction::backfill()
        .notes()
        .periods([PERIOD])
        .sec_client(sec_client)
        .repository(repository)
        .retry_delays([])
        .run(|_, _| {})
        .await
        .success_count();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_the_last_503_when_every_attempt_gets_a_503() {
    let sec_client = always_unavailable_client();
    let request = notes_request();

    let expected_result = Some(FailedSecRequest::from(InvalidSecResponse::new(
        ErrorReason::InvalidStatusCode {
            status_code: StatusCode::ServiceUnavailable,
        },
    )));

    let result = sec_client.execute_sec_request(request).await.err();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_send_the_maximum_attempts_when_every_attempt_gets_a_503() {
    let sec_client = always_unavailable_client();
    let request = notes_request();

    let expected_result = BTreeMap::from([(NOTES_URL.to_string(), 3)]);

    let _ = sec_client.execute_sec_request(request).await;
    let result = sec_client.inner.request_counts.per_key();

    assert_eq!(result, expected_result);
}
