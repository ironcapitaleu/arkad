//! # Store Filing Index State
//!
//! Provides [`StoreFilingIndex`], the [`StoreSecResponse`] state of the [`FilingIndex`] kind,
//! which writes the raw body of the folder `index.json` of one filing to the store and outputs
//! the key of the stored document.
//!
//! The document holds the body exactly as the SEC sent it. It lists the files of the folder, so a
//! later run can tell from the store whether the folder holds an XBRL instance.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`data`]: The [`StoreFilingIndexOutput`] data type.
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract::store_sec_response`]: The generic store state.
//! - [`crate::shared::filing_documents_ref::filing_index_key`]: The key of the stored document.

use crate::implementations::states::extract::store_sec_response::{
    StoreSecResponse, StoreSecResponseContext, StoreSecResponseInput,
};
use crate::shared::filing_documents_ref::filing_index_key;
use crate::shared::filing_ref::FilingRef;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::RawFilingIndex;
use crate::shared::request::implementations::sec_request::{FilingIndex, StoredAs};
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::shared::run_fields::RunFields;
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;

pub mod constants;
pub mod data;

pub use constants::STATE_NAME;
pub use data::StoreFilingIndexOutput;

/// The [`StoreSecResponse`] state of the [`FilingIndex`] kind.
///
/// It writes one [`RawDocument<RawFilingIndex>`] under the key
/// `sec/filings/CIK##########/<accession number>/index.json` and outputs that key.
pub type StoreFilingIndex<R> = StoreSecResponse<FilingIndex, R>;

/// The input of [`StoreFilingIndex`].
pub type StoreFilingIndexInput = StoreSecResponseInput<FilingIndex>;

/// The context of [`StoreFilingIndex`].
pub type StoreFilingIndexContext<R> = StoreSecResponseContext<FilingIndex, R>;

/// Builds the document with [`RawDocument::from_response`], which keeps the JSON body unchanged.
impl StoredAs for FilingIndex {
    type Source = RawFilingIndex;
    type Output = StoreFilingIndexOutput;

    const STATE_NAME: &'static str = STATE_NAME;

    fn store(
        response: &SecResponse,
        filing: &FilingRef,
        requested_url: &Url,
        user_agent: &UserAgent,
        run: RunFields,
    ) -> (RawDocument<RawFilingIndex>, StoreFilingIndexOutput) {
        let key = filing_index_key(filing);
        let document =
            RawDocument::from_response(key.clone(), response, requested_url, user_agent, run);
        (document, StoreFilingIndexOutput::new(key))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use serde::Serialize;
    use state_maschine::prelude::State as SMState;
    use storage::implementations::repository::filesystem::{
        Document, DocumentKey, FilesystemRepository, IngestionMode, ReadWrite,
    };
    use uuid::Uuid;

    use super::*;
    use crate::shared::build_info::producer;
    use crate::shared::cik::Cik;
    use crate::shared::content_type::ContentType;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::headers::Headers;
    use crate::shared::raw_document::RawDocumentMetadataContext;
    use crate::shared::status_code::StatusCode;
    use crate::tests::fixtures::sample_write_repository::FakeFilingIndexRepository;
    use crate::traits::state_machine::state::State;

    // Extra whitespace that a parse and a reserialization drop.
    const RAW_BODY: &[u8] =
        b"{ \"directory\": { \"item\": [{ \"name\": \"alco-20260908_htm.xml\" }] } }";

    // The mode of the new-filing path. A context of `Incremental` shows that this state reads the
    // mode from its context.
    const INGESTION_MODE: IngestionMode = IngestionMode::Incremental;

    const EXECUTION_ID: Uuid = Uuid::from_u128(0x0b6f_3c1e_5d2a_4f7b_9c3e_8a1d_2b4c_6e8f);

    fn requested_url() -> Url {
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn redirected_url() -> Url {
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/redirected/index.json"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn user_agent() -> UserAgent {
        UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed")
    }

    /// Returns an input whose response came from a redirect, so the two URLs differ.
    fn sample_input() -> StoreFilingIndexInput {
        let response = SecResponse::from_parts(
            redirected_url(),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            RAW_BODY.to_vec(),
        )
        .expect("Given a hardcoded valid JSON body, building the response should always succeed");
        StoreFilingIndexInput::new(response, requested_url(), user_agent())
    }

    fn sample_state_in_mode(
        ingestion_mode: IngestionMode,
        repository: FakeFilingIndexRepository,
    ) -> StoreFilingIndex<FakeFilingIndexRepository> {
        let filing = FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenQ, false),
            NaiveDate::from_ymd_opt(2026, 9, 8)
                .expect("Given a hardcoded calendar date, the date should always build"),
        );
        let context =
            StoreFilingIndexContext::new(filing, EXECUTION_ID, ingestion_mode, repository);
        StoreFilingIndex::new(sample_input(), context)
    }

    fn sample_state(
        repository: FakeFilingIndexRepository,
    ) -> StoreFilingIndex<FakeFilingIndexRepository> {
        sample_state_in_mode(INGESTION_MODE, repository)
    }

    /// Runs the state against an accepting fake and returns the one document it persisted.
    async fn persisted_document() -> RawDocument<RawFilingIndex> {
        let repository = FakeFilingIndexRepository::new();
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
        DocumentKey::new("sec/filings/CIK0000003545/0000003545-26-000047/index.json")
            .expect("Given a hardcoded valid relative path, the key should always build")
    }

    #[test]
    fn should_return_state_name_for_store_filing_index() {
        let state = sample_state(FakeFilingIndexRepository::new());

        let expected_result = "Store Filing Index";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_filing_index_key_when_the_write_succeeds() {
        let mut state = sample_state(FakeFilingIndexRepository::new());

        let expected_result = Some(&StoreFilingIndexOutput::new(sample_key()));

        state
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_persist_the_document_under_the_filing_index_key() {
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
    async fn should_persist_raw_filing_index_as_the_document_type() {
        let document = persisted_document().await;

        let expected_result = "raw_filing_index";

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

    const fn implements_state<T: State + Serialize>() {}
    #[test]
    const fn should_implement_state_for_store_filing_index_of_a_filesystem_repository() {
        implements_state::<
            StoreFilingIndex<FilesystemRepository<ReadWrite, RawDocument<RawFilingIndex>>>,
        >();
    }
}
