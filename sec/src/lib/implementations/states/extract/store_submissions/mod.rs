//! # Store Submissions State
//!
//! Provides [`StoreSubmissions`], the [`StoreSecResponse`] state of the [`Submissions`] kind,
//! which writes the raw body of an SEC `submissions` response to the store and outputs the key of
//! the stored document.
//!
//! The document holds the body exactly as the SEC sent it, because a parse and a reserialization
//! change whitespace, number format, and duplicate keys.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`], and the parts of the document key.
//! - [`data`]: The [`StoreSubmissionsOutput`] data type.
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract::store_sec_response`]: The generic store state.
//! - [`crate::shared::raw_document`]: The [`RawDocument`] type the state writes.

use storage::implementations::repository::filesystem::DocumentKey;

use crate::implementations::states::extract::store_sec_response::{
    StoreSecResponse, StoreSecResponseContext, StoreSecResponseInput,
};
use crate::shared::cik::Cik;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::RawSubmissions;
use crate::shared::request::implementations::sec_request::{StoredAs, Submissions};
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::shared::run_fields::RunFields;
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;

pub mod constants;
pub mod data;

pub use constants::{STATE_NAME, SUBMISSIONS_KEY_PREFIX, SUBMISSIONS_KEY_SUFFIX};
pub use data::StoreSubmissionsOutput;

/// The [`StoreSecResponse`] state of the [`Submissions`] kind.
///
/// It writes one [`RawDocument<RawSubmissions>`] under the key
/// `sec/submissions/CIK##########.json` and outputs that [`DocumentKey`].
pub type StoreSubmissions<R> = StoreSecResponse<Submissions, R>;

/// The input of [`StoreSubmissions`].
pub type StoreSubmissionsInput = StoreSecResponseInput<Submissions>;

/// The context of [`StoreSubmissions`].
pub type StoreSubmissionsContext<R> = StoreSecResponseContext<Submissions, R>;

/// Builds the document with [`RawDocument::from_response`], which keeps the JSON body unchanged.
impl StoredAs for Submissions {
    type Source = RawSubmissions;
    type Output = StoreSubmissionsOutput;

    const STATE_NAME: &'static str = STATE_NAME;
    const INPUT_LABEL: &'static str = "CIK";

    fn store(
        response: &SecResponse,
        cik: &Cik,
        requested_url: &Url,
        user_agent: &UserAgent,
        run: RunFields,
    ) -> (RawDocument<RawSubmissions>, StoreSubmissionsOutput) {
        let key = submissions_key(cik);
        let document =
            RawDocument::from_response(key.clone(), response, requested_url, user_agent, run);
        (document, StoreSubmissionsOutput::new(key))
    }
}

/// Returns the key `sec/submissions/CIK##########.json` of the `submissions` JSON of a company.
fn submissions_key(cik: &Cik) -> DocumentKey {
    DocumentKey::new(format!(
        "{SUBMISSIONS_KEY_PREFIX}{cik}{SUBMISSIONS_KEY_SUFFIX}"
    ))
    .expect("Given a CIK of 10 digits, the relative key path should always be valid")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use serde::Serialize;
    use state_maschine::prelude::State as SMState;
    use storage::implementations::repository::filesystem::{
        Document, FilesystemRepository, IngestionMode, ReadWrite,
    };
    use storage::{Sha256Digest, WriteError};
    use uuid::Uuid;

    use super::*;
    use crate::error::State as StateError;
    use crate::error::state_machine::state::FailedDocumentWrite;
    use crate::shared::build_info::producer;
    use crate::shared::content_type::ContentType;
    use crate::shared::document_storage::{DocumentStorageErrorReason, FailedDocumentStorage};
    use crate::shared::headers::Headers;
    use crate::shared::raw_document::RawDocumentMetadataContext;
    use crate::shared::status_code::StatusCode;
    use crate::tests::fixtures::sample_write_repository::FakeSubmissionsRepository;
    use crate::traits::state_machine::state::State;

    // Extra whitespace that a parse and a reserialization drop.
    const RAW_BODY: &[u8] = b"{ \"cik\": \"0001067983\",  \"name\": \"BERKSHIRE HATHAWAY INC\" }";

    const CONFLICT_REASON: &str =
        "A document already exists under sec/submissions/CIK0001067983.json";

    // The mode of the new-filing path. The Company Facts super-state fixes `Backfill`, so a
    // context of `Incremental` shows that this state reads the mode from its context.
    const INGESTION_MODE: IngestionMode = IngestionMode::Incremental;

    const EXECUTION_ID: Uuid = Uuid::from_u128(0x0b6f_3c1e_5d2a_4f7b_9c3e_8a1d_2b4c_6e8f);

    fn requested_url() -> Url {
        "https://data.sec.gov/submissions/CIK0001067983.json"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn redirected_url() -> Url {
        "https://data.sec.gov/submissions/redirected/CIK0001067983.json"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn user_agent() -> UserAgent {
        UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed")
    }

    /// Returns an input whose response came from a redirect, so the two URLs differ.
    fn sample_input() -> StoreSubmissionsInput {
        let response = SecResponse::from_parts(
            redirected_url(),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            RAW_BODY.to_vec(),
        )
        .expect("Given a hardcoded valid JSON body, building the response should always succeed");
        StoreSubmissionsInput::new(response, requested_url(), user_agent())
    }

    fn sample_state_in_mode(
        ingestion_mode: IngestionMode,
        repository: FakeSubmissionsRepository,
    ) -> StoreSubmissions<FakeSubmissionsRepository> {
        let cik = Cik::new("1067983")
            .expect("Given a hardcoded numeric CIK, its creation should always succeed");
        let context = StoreSubmissionsContext::new(cik, EXECUTION_ID, ingestion_mode, repository);
        StoreSubmissions::new(sample_input(), context)
    }

    fn sample_state(
        repository: FakeSubmissionsRepository,
    ) -> StoreSubmissions<FakeSubmissionsRepository> {
        sample_state_in_mode(INGESTION_MODE, repository)
    }

    /// Runs the state against an accepting fake and returns the one document it persisted.
    async fn persisted_document() -> RawDocument<RawSubmissions> {
        let repository = FakeSubmissionsRepository::new();
        let mut state = sample_state(repository.clone());
        state
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");
        repository
            .persisted()
            .pop()
            .expect("Given a successful computation, the fake should hold the persisted document")
    }

    fn sample_key() -> DocumentKey {
        DocumentKey::new("sec/submissions/CIK0001067983.json")
            .expect("Given a hardcoded valid relative path, the key should always build")
    }

    #[test]
    fn should_return_state_name_for_store_submissions() {
        let state = sample_state(FakeSubmissionsRepository::new());

        let expected_result = "Store Submissions";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_input_data_when_in_initial_state() {
        let state = sample_state(FakeSubmissionsRepository::new());

        let expected_result = &sample_input();

        let result = state.input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_have_no_output_data_when_creating_new_state() {
        let state = sample_state(FakeSubmissionsRepository::new());

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_submissions_key_when_the_write_succeeds() {
        let mut state = sample_state(FakeSubmissionsRepository::new());

        let expected_result = Some(&StoreSubmissionsOutput::new(sample_key()));

        state
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_persist_one_document_when_the_write_succeeds() {
        let repository = FakeSubmissionsRepository::new();
        let mut state = sample_state(repository.clone());

        let expected_result = 1;

        state
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");
        let result = repository.persisted().len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_persist_the_document_under_the_submissions_key() {
        let document = persisted_document().await;

        let expected_result = &sample_key();

        let result = document.key();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_persist_the_raw_body_unchanged() {
        let document = persisted_document().await;

        let expected_result = RAW_BODY;

        let result = document.bytes();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_persist_a_size_that_matches_the_raw_body() {
        let document = persisted_document().await;

        let expected_result = RAW_BODY.len() as u64;

        let result = document.metadata().size_in_bytes();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_persist_a_digest_that_matches_the_raw_body() {
        let document = persisted_document().await;

        let expected_result = &Sha256Digest::of(RAW_BODY);

        let result = document.metadata().sha256();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_persist_the_context_of_the_fetch_when_the_response_was_redirected() {
        let document = persisted_document().await;

        let expected_result = &RawDocumentMetadataContext {
            requested_url: requested_url(),
            final_url: redirected_url(),
            http_status: StatusCode::Ok,
            user_agent: user_agent(),
            filing: None,
        };

        let result = document.metadata().context();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_persist_the_execution_id_of_the_context() {
        let document = persisted_document().await;

        let expected_result = EXECUTION_ID;

        let result = document.metadata().execution_id();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_persist_the_ingestion_mode_of_the_context() {
        let document = persisted_document().await;

        let expected_result = INGESTION_MODE;

        let result = document.metadata().ingestion_mode();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_persist_backfill_as_the_ingestion_mode_when_the_context_names_backfill() {
        let repository = FakeSubmissionsRepository::new();
        let mut state = sample_state_in_mode(IngestionMode::Backfill, repository.clone());

        let expected_result = vec![IngestionMode::Backfill];

        state
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");
        let result: Vec<IngestionMode> = repository
            .persisted()
            .iter()
            .map(|document| document.metadata().ingestion_mode())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_persist_raw_submissions_as_the_document_type() {
        let document = persisted_document().await;

        let expected_result = "raw_submissions";

        let result = document.metadata().document_type().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_name_this_state_as_the_producer_of_the_document() {
        let document = persisted_document().await;

        let expected_result = &producer(STATE_NAME);

        let result = document.metadata().producer();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_document_write_when_the_repository_rejects_the_write() {
        let write_error = WriteError::conflicting_write(CONFLICT_REASON);
        let mut state = sample_state(FakeSubmissionsRepository::rejecting(write_error));

        let expected_result = Err(StateError::FailedDocumentWrite(FailedDocumentWrite::new(
            STATE_NAME,
            FailedDocumentStorage::new(DocumentStorageErrorReason::ConflictingWrite {
                details: CONFLICT_REASON.to_string(),
            }),
        )));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_have_no_output_data_when_the_repository_rejects_the_write() {
        let write_error = WriteError::conflicting_write(CONFLICT_REASON);
        let mut state = sample_state(FakeSubmissionsRepository::rejecting(write_error));

        let expected_result = false;

        let _ = state.compute_output_data_async().await;
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_compute_output_data_when_called_through_the_blocking_wrapper() {
        let mut state = sample_state(FakeSubmissionsRepository::new());

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_no_output_before_computation() {
        let state = sample_state(FakeSubmissionsRepository::new());

        let expected_result = true;

        let result = state.to_string().ends_with("Output Data:\n\tNone");

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_store_submissions() {
        implements_auto_traits::<StoreSubmissions<FakeSubmissionsRepository>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_be_thread_safe_for_store_submissions() {
        implements_send::<StoreSubmissions<FakeSubmissionsRepository>>();
        implements_sync::<StoreSubmissions<FakeSubmissionsRepository>>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_submissions() {
        implements_value_traits::<StoreSubmissions<FakeSubmissionsRepository>>();
    }

    const fn implements_state<T: State + Serialize>() {}
    #[test]
    const fn should_implement_state_for_store_submissions_of_a_filesystem_repository() {
        implements_state::<
            StoreSubmissions<FilesystemRepository<ReadWrite, RawDocument<RawSubmissions>>>,
        >();
    }
}
