//! # `ExtractFilingSuperState` Integration Tests
//!
//! Runs the filing machine against the live SEC archive for real filings, and checks the files it
//! stores and the reference it returns.
//!
//! ## External Dependencies
//!
//! - `www.sec.gov`: the folder index, the XBRL instance, and the label file of each filing below

use std::path::Path;

use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use storage::Sha256Digest;
use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
use tempfile::TempDir;
use uuid::Uuid;

use sec::implementations::states::extract::filing::ExtractFilingSuperState;
use sec::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
use sec::shared::cik::Cik;
use sec::shared::filing_documents_ref::FilingDocumentsRef;
use sec::shared::filing_ref::FilingRef;
use sec::shared::financial::accession_number::AccessionNumber;
use sec::shared::financial::form::{Form, ParsedForm};
use sec::shared::http_client::implementations::reqwest_client::ReqwestClient;
use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::{RawXbrlInstance, RawXbrlLabels};
use sec::shared::request::implementations::sec_request::FilingIndex;
use sec::shared::user_agent::UserAgent;

type InstanceRepository = FilesystemRepository<Write, RawDocument<RawXbrlInstance>>;
type LabelsRepository = FilesystemRepository<Write, RawDocument<RawXbrlLabels>>;
type FilingSuperState = ExtractFilingSuperState<
    PrepareSecRequest<FilingIndex>,
    InstanceRepository,
    LabelsRepository,
    SecClient,
>;

/// The folder of the inline XBRL 10-Q of Alico (CIK 3545) for the quarter to 2026-06-30.
const INLINE_XBRL_FOLDER: &str = "sec/filings/CIK0000003545/0000003545-26-000043";

/// The folder of the 10-K of Apple (CIK 320193) for 2017, filed without inline XBRL.
const PLAIN_XBRL_FOLDER: &str = "sec/filings/CIK0000320193/0000320193-17-000070";

/// Creates an `SecClient` that sends real requests to the SEC.
fn sec_client() -> SecClient {
    let user_agent = UserAgent::new("ArkadTest admin@example.com")
        .expect("A hardcoded SEC-format string should always validate");
    let http_client = ReqwestClient::new(user_agent)
        .expect("Given a validated user agent, building the HTTP client fails only if the TLS backend cannot initialize");
    SecClient::new(http_client)
}

fn filing_ref(cik: &str, accession_number: &str, form: Form, filed: (i32, u32, u32)) -> FilingRef {
    FilingRef::new(
        Cik::new(cik).expect("Given a hardcoded valid CIK, parsing should always succeed"),
        AccessionNumber::new(accession_number).expect(
            "Given a hardcoded accession number of the right shape, creation should always succeed",
        ),
        ParsedForm::new(form, false),
        NaiveDate::from_ymd_opt(filed.0, filed.1, filed.2)
            .expect("Given a hardcoded calendar date, the date should always build"),
    )
}

fn inline_xbrl_10q() -> FilingRef {
    filing_ref("3545", "0000003545-26-000043", Form::TenQ, (2026, 8, 10))
}

fn plain_xbrl_10k() -> FilingRef {
    filing_ref("320193", "0000320193-17-000070", Form::TenK, (2017, 11, 3))
}

fn temporary_root() -> TempDir {
    TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    )
}

/// Builds the machine for `filing`, with both repositories under `root`.
fn super_state(filing: FilingRef, root: &Path) -> FilingSuperState {
    ExtractFilingSuperState::builder()
        .filing_ref(filing)
        .sec_client(sec_client())
        .execution_id(Uuid::new_v4())
        .instance_repository(
            FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<RawXbrlInstance>>()
                .root(root)
                .build(),
        )
        .labels_repository(
            FilesystemRepository::builder()
                .writer()
                .document_type::<RawDocument<RawXbrlLabels>>()
                .root(root)
                .build(),
        )
        .build()
}

/// Returns the size and the SHA-256 of the stored file `file_name` in `folder` under `root`.
fn stored_size_and_sha256(root: &Path, folder: &str, file_name: &str) -> (usize, String) {
    let bytes = std::fs::read(root.join(folder).join(file_name))
        .expect("Given a run that stored the file, reading it back should always succeed");
    (bytes.len(), Sha256Digest::of(&bytes).to_string())
}

#[tokio::test]
async fn should_return_the_filing_documents_ref_when_extracting_an_inline_xbrl_10q() {
    let root = temporary_root();
    let filing = inline_xbrl_10q();

    let expected_result = Ok(FilingDocumentsRef::new(inline_xbrl_10q(), true));

    let result = super_state(filing, root.path())
        .run_to_documents(|_| {})
        .await
        .map_err(|error| error.to_string());

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_store_the_htm_xml_instance_when_extracting_an_inline_xbrl_10q() {
    let root = temporary_root();
    let filing = inline_xbrl_10q();

    let expected_result = (
        1_010_807,
        "6980d4f56c4e900804ead5bc8ef899a9989b032e902e44ddd6eaa397593a14b8".to_string(),
    );

    super_state(filing, root.path())
        .run_to_documents(|_| {})
        .await
        .expect("Given a published filing with an instance, the run should always succeed");
    let result = stored_size_and_sha256(root.path(), INLINE_XBRL_FOLDER, "instance.xml");

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_store_the_label_file_when_extracting_an_inline_xbrl_10q() {
    let root = temporary_root();
    let filing = inline_xbrl_10q();

    let expected_result = (
        639_124,
        "bce223dacc2c6c708574c281466a15e0bc020be9abef438d178a1285d56a74ed".to_string(),
    );

    super_state(filing, root.path())
        .run_to_documents(|_| {})
        .await
        .expect("Given a published filing with a label file, the run should always succeed");
    let result = stored_size_and_sha256(root.path(), INLINE_XBRL_FOLDER, "lab.xml");

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_store_the_plain_xml_instance_when_the_filing_has_no_inline_xbrl() {
    let root = temporary_root();
    let filing = plain_xbrl_10k();

    let expected_result = (
        3_842_300,
        "0f11014185252311803c6cf72e6841cfc543de6e4b35de51c8b2d7fd6d46496f".to_string(),
    );

    super_state(filing, root.path())
        .run_to_documents(|_| {})
        .await
        .expect(
            "Given a published filing with a plain XBRL instance, the run should always succeed",
        );
    let result = stored_size_and_sha256(root.path(), PLAIN_XBRL_FOLDER, "instance.xml");

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_store_the_label_file_when_the_filing_has_no_inline_xbrl() {
    let root = temporary_root();
    let filing = plain_xbrl_10k();

    let expected_result = (
        930_710,
        "3cac6dd367351df21e2d48c02e23c654a5052696a65ade6ce893b26bb259cc8d".to_string(),
    );

    super_state(filing, root.path())
        .run_to_documents(|_| {})
        .await
        .expect("Given a published filing with a label file, the run should always succeed");
    let result = stored_size_and_sha256(root.path(), PLAIN_XBRL_FOLDER, "lab.xml");

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_end_with_the_error_of_the_folder_index_request_when_the_filing_does_not_exist() {
    let root = temporary_root();
    let filing = filing_ref("3545", "0000003545-99-999999", Form::TenQ, (2026, 8, 10));

    let expected_result = Some(("Execute SEC Request".to_string(), "Problem occurred during internal state operations: '[StateError] A state level error occurred, Caused by: [FailedRequestExecution] Failure in State: 'Execute SEC Request', Caused by: [FailedSecRequest] SEC request failed, Caused by: [InvalidResponse] Response validation failed, Caused by: [InvalidSecResponse] Invalid SEC Response, Reason: 'Status code '404' is not accepted''".to_string()));

    let result = super_state(filing, root.path())
        .run_to_documents(|_| {})
        .await
        .err()
        .map(|error| (error.state_name, error.source.to_string()));

    assert_eq!(result, expected_result);
}
