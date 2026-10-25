//! # Store Company Facts State
//!
//! Provides the [`StoreCompanyFacts`] state, which writes the raw body of an SEC Company Facts
//! response to the store and outputs the key of the stored document.
//!
//! The state writes through the [`WriteRepository`](storage::WriteRepository) port that its
//! context holds, so it names no concrete backend. The document holds the body exactly as the SEC
//! sent it, because a parse and a reserialization change whitespace, number format, and duplicate
//! keys.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`], and the parts of the document key.
//! - [`context`]: The [`StoreCompanyFactsContext`] carried alongside the state.
//! - [`data`]: The [`StoreCompanyFactsInput`] and [`StoreCompanyFactsOutput`] data types.
//!
//! ## See Also
//!
//! - [`crate::shared::raw_document`]: The [`RawDocument`] type the state writes.
//! - [`crate::traits::state_machine::state::State`]: The trait implemented by [`StoreCompanyFacts`].

use std::fmt;

use async_trait::async_trait;
use chrono::Utc;
use serde::Serialize;
use state_maschine::prelude::State as SMState;
use storage::implementations::repository::filesystem::DocumentKey;

use crate::error::State as StateError;
use crate::error::state_machine::state::FailedDocumentWrite;
use crate::shared::build_info::producer;
use crate::shared::cik::Cik;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::RawCompanyFacts;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::{COMPANY_FACTS_KEY_PREFIX, COMPANY_FACTS_KEY_SUFFIX, STATE_NAME};
pub use context::{RawCompanyFactsRepository, StoreCompanyFactsContext};
pub use data::StoreCompanyFactsInput;
pub use data::StoreCompanyFactsOutput;

/// Writes the raw body of an SEC Company Facts response to the store.
///
/// Builds one [`RawDocument<RawCompanyFacts>`] with
/// [`RawDocument::from_response`], writes it under the key `sec/companyfacts/CIK##########.json`
/// through the repository of type `R`, and outputs that [`DocumentKey`].
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
// The context leaves out the repository, so serialization needs no bound on `R`.
#[serde(bound(serialize = ""))]
pub struct StoreCompanyFacts<R> {
    input: StoreCompanyFactsInput,
    context: StoreCompanyFactsContext<R>,
    output: Option<StoreCompanyFactsOutput>,
}

impl<R> StoreCompanyFacts<R> {
    /// Creates a new [`StoreCompanyFacts`] state from its input and context, with no output
    /// computed yet.
    #[must_use]
    pub const fn new(input: StoreCompanyFactsInput, context: StoreCompanyFactsContext<R>) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        StoreCompanyFactsInput,
        Option<StoreCompanyFactsOutput>,
        StoreCompanyFactsContext<R>,
    ) {
        (self.input, self.output, self.context)
    }
}

/// Returns the key `sec/companyfacts/CIK##########.json` of the Company Facts of a company.
fn company_facts_key(cik: &Cik) -> DocumentKey {
    DocumentKey::new(format!(
        "{COMPANY_FACTS_KEY_PREFIX}{cik}{COMPANY_FACTS_KEY_SUFFIX}"
    ))
    .expect("Given a CIK of 10 digits, the relative key path should always be valid")
}

#[async_trait]
impl<R: RawCompanyFactsRepository> State for StoreCompanyFacts<R> {
    /// Writes the raw document to the repository and stores its key as output.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::FailedDocumentWrite`] when the repository rejects the write.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let key = company_facts_key(&self.context.cik);
        let document = RawDocument::<RawCompanyFacts>::from_response(
            key.clone(),
            &self.input.response,
            &self.input.requested_url,
            &self.input.user_agent,
            Utc::now(),
            self.context.execution_id,
            producer(STATE_NAME),
        );

        match self.context.repository.persist(document).await {
            Ok(()) => {
                self.output = Some(StoreCompanyFactsOutput::new(key));
                Ok(())
            }
            Err(e) => {
                let e: StateError =
                    FailedDocumentWrite::new(self.state_name().to_string(), e).into();
                Err(e)
            }
        }
    }
}

impl<R: RawCompanyFactsRepository> SMState for StoreCompanyFacts<R> {
    type InputData = StoreCompanyFactsInput;
    type OutputData = StoreCompanyFactsOutput;
    type Context = StoreCompanyFactsContext<R>;

    /// Returns the human-readable name of this state.
    fn state_name(&self) -> impl ToString {
        STATE_NAME
    }

    /// Blocking wrapper around [`compute_output_data_async`](crate::traits::state_machine::state::State::compute_output_data_async).
    ///
    /// Detects whether a tokio runtime is available and runs the async computation
    /// synchronously. This allows SEC states to be used as regular `SMState` implementations.
    ///
    /// # Panics
    /// - Panics if the async computation returns an error.
    /// - Panics if called inside a `current_thread` runtime (`block_in_place`
    ///   requires a multi-thread runtime). All binaries and tests in this project
    ///   use `flavor = "multi_thread"`.
    fn compute_output_data(&mut self) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(self.compute_output_data_async()))
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.compute_output_data_async())
        };

        if let Err(e) = result {
            let state_err: StateError = e;
            panic!("compute_output_data failed: {state_err}")
        }
    }

    fn context_data(&self) -> &Self::Context {
        &self.context
    }

    fn input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl<R: RawCompanyFactsRepository> fmt::Display for StoreCompanyFacts<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use storage::implementations::repository::filesystem::{
        Document, FilesystemRepository, ReadWrite,
    };
    use storage::{Sha256Digest, WriteError};
    use uuid::Uuid;

    use super::*;
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::raw_document::RawDocumentMetadataContext;
    use crate::shared::response::implementations::sec_response::SecResponse;
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    // Whitespace and a trailing zero that a parse and a reserialization drop.
    const RAW_BODY: &[u8] = b"{ \"cik\": 1067983, \"price\": 1.50 }";

    const EXECUTION_ID: Uuid = Uuid::from_u128(0x0b6f_3c1e_5d2a_4f7b_9c3e_8a1d_2b4c_6e8f);

    fn requested_url() -> Url {
        "https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn sample_input() -> StoreCompanyFactsInput {
        let response = SecResponse::from_parts(
            requested_url(),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            RAW_BODY.to_vec(),
        )
        .expect("Given a hardcoded valid JSON body, building the response should always succeed");
        let user_agent = UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        StoreCompanyFactsInput::new(response, requested_url(), user_agent)
    }

    fn sample_state(repository: FakeWriteRepository) -> StoreCompanyFacts<FakeWriteRepository> {
        let cik = Cik::new("1067983")
            .expect("Given a hardcoded numeric CIK, its creation should always succeed");
        let context = StoreCompanyFactsContext::new(cik, EXECUTION_ID, repository);
        StoreCompanyFacts::new(sample_input(), context)
    }

    /// Runs the state against an accepting fake and returns the one document it persisted.
    async fn persisted_document() -> RawDocument<RawCompanyFacts> {
        let repository = FakeWriteRepository::new();
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
        DocumentKey::new("sec/companyfacts/CIK0001067983.json")
            .expect("Given a hardcoded valid relative path, the key should always build")
    }

    #[test]
    fn should_return_state_name_for_store_company_facts() {
        let state = sample_state(FakeWriteRepository::new());

        let expected_result = "Store Company Facts";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_input_data_when_in_initial_state() {
        let state = sample_state(FakeWriteRepository::new());

        let expected_result = &sample_input();

        let result = state.input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_have_no_output_data_when_creating_new_state() {
        let state = sample_state(FakeWriteRepository::new());

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_company_facts_key_when_the_write_succeeds() {
        let mut state = sample_state(FakeWriteRepository::new());

        let expected_result = Some(&StoreCompanyFactsOutput::new(sample_key()));

        state
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_persist_one_document_when_the_write_succeeds() {
        let repository = FakeWriteRepository::new();
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
    async fn should_persist_the_document_under_the_company_facts_key() {
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
    async fn should_persist_the_context_of_the_fetch() {
        let document = persisted_document().await;
        let input = sample_input();

        let expected_result = &RawDocumentMetadataContext::new(
            input.response(),
            input.requested_url(),
            input.user_agent(),
        );

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
    async fn should_name_this_state_as_the_producer_of_the_document() {
        let document = persisted_document().await;

        let expected_result = &producer(STATE_NAME);

        let result = document.metadata().producer();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_document_write_when_the_repository_rejects_the_write() {
        let write_error = WriteError::conflicting_write("stored bytes differ");
        let mut state = sample_state(FakeWriteRepository::rejecting(write_error.clone()));

        let expected_result = Err(StateError::FailedDocumentWrite(FailedDocumentWrite::new(
            STATE_NAME,
            write_error,
        )));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_have_no_output_data_when_the_repository_rejects_the_write() {
        let write_error = WriteError::conflicting_write("stored bytes differ");
        let mut state = sample_state(FakeWriteRepository::rejecting(write_error));

        let expected_result = false;

        let _ = state.compute_output_data_async().await;
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_compute_output_data_when_called_through_the_blocking_wrapper() {
        let mut state = sample_state(FakeWriteRepository::new());

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_no_output_before_computation() {
        let state = sample_state(FakeWriteRepository::new());

        let expected_result = true;

        let result = state.to_string().ends_with("Output Data:\n\tNone");

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_store_company_facts() {
        implements_auto_traits::<StoreCompanyFacts<FakeWriteRepository>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_be_thread_safe_for_store_company_facts() {
        implements_send::<StoreCompanyFacts<FakeWriteRepository>>();
        implements_sync::<StoreCompanyFacts<FakeWriteRepository>>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_company_facts() {
        implements_value_traits::<StoreCompanyFacts<FakeWriteRepository>>();
    }

    const fn implements_state<T: State + Serialize>() {}
    #[test]
    const fn should_implement_state_for_store_company_facts_of_a_filesystem_repository() {
        implements_state::<
            StoreCompanyFacts<FilesystemRepository<ReadWrite, RawDocument<RawCompanyFacts>>>,
        >();
    }
}
