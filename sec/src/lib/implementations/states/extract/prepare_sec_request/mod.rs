//! # Prepare SEC Request State
//!
//! Provides the [`PrepareSecRequest`] state, which builds the [`SecRequest`] of a request kind from
//! its input.
//!
//! The kind `K` sets the input and the state name. For example, [`NotesFile`] takes a
//! [`NotesPeriod`](crate::shared::notes_period::NotesPeriod) and names its state "Prepare Notes
//! Request". The state builds the request with [`SecRequestTrait::new`], so it needs no code for a
//! single kind. Preparation is local and synchronous: it makes no network call.
//!
//! ## Modules
//!
//! - [`builder`]: The typestate [`PrepareSecRequestBuilder`].
//! - [`constants`]: The [`STATE_NAME`] of the [`CompanyFacts`] kind.
//! - [`context`]: The [`PrepareSecRequestContext`] carried alongside the state.
//! - [`data`]: The [`PrepareSecRequestInput`] and [`PrepareSecRequestOutput`] data types.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::prepare_sec_request::*;
//! use sec::prelude::*;
//! use sec::shared::cik::Cik;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
//! # use sec::shared::raw_document::RawDocument;
//! # use sec::shared::raw_source::RawCompanyFacts;
//! # use storage::implementations::repository::filesystem::FilesystemRepository;
//! # let repository = FilesystemRepository::builder()
//! #     .writer()
//! #     .document_type::<RawDocument<RawCompanyFacts>>()
//! #     .root("data")
//! #     .build();
//! let mut state = PrepareSecRequest::builder()
//!     .all_company_facts()
//!     .cik(cik)
//!     .repository(repository)
//!     .build();
//! state
//!     .compute_output_data_async()
//!     .await
//!     .expect("Request preparation is infallible for a valid CIK");
//!
//! let expected_result = true;
//!
//! let result = state.output_data().is_some();
//!
//! assert_eq!(result, expected_result);
//! # }
//! ```
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract`]: Parent module describing the extraction flow.
//! - [`crate::shared::request::implementations::sec_request::SecRequest`]: The request type produced by this state.
//! - [`crate::traits::state_machine::state::State`]: The trait implemented by [`PrepareSecRequest`].

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::shared::raw_repository::RawRepository;
use crate::shared::request::SecRequest as SecRequestTrait;
use crate::shared::request::implementations::sec_request::{
    CompanyFacts, NotesFile, RequestKind, SecRequest, StoredAs,
};
use crate::traits::state_machine::state::State;

pub mod builder;
pub mod constants;
pub mod context;
pub mod data;

pub use builder::PrepareSecRequestBuilder;
pub use constants::STATE_NAME;
pub use context::PrepareSecRequestContext;
pub use data::PrepareSecRequestInput;
pub use data::PrepareSecRequestOutput;

/// Builds the [`SecRequest`] of the request kind `K` from its input.
///
/// The context holds the storage repository `R`.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
#[serde(bound(serialize = "R: Serialize"))]
pub struct PrepareSecRequest<K: RequestKind, R> {
    input: PrepareSecRequestInput<K>,
    context: PrepareSecRequestContext<K, R>,
    output: Option<PrepareSecRequestOutput<K>>,
}

/// Alias of [`PrepareSecRequest`] for the [`CompanyFacts`] kind.
///
/// It is the same type under a shorter name, not a separate state.
pub type PrepareCompanyFactsRequest<R> = PrepareSecRequest<CompanyFacts, R>;

/// Alias of [`PrepareSecRequest`] for the [`NotesFile`] kind.
///
/// It is the same type under a shorter name, not a separate state.
pub type PrepareNotesRequest<R> = PrepareSecRequest<NotesFile, R>;

impl<K: RequestKind, R> PrepareSecRequest<K, R> {
    /// Creates a new [`PrepareSecRequest`] state from its input and context, with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::prepare_sec_request::*;
    /// use sec::shared::cik::Cik;
    /// use sec::shared::request::implementations::sec_request::CompanyFacts;
    /// use state_maschine::prelude::State as SMState;
    ///
    /// let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
    /// let input = PrepareSecRequestInput::<CompanyFacts>::new(cik.clone());
    /// # use sec::shared::raw_document::RawDocument;
    /// # use sec::shared::raw_source::RawCompanyFacts;
    /// # use storage::implementations::repository::filesystem::FilesystemRepository;
    /// # let repository = FilesystemRepository::builder()
    /// #     .writer()
    /// #     .document_type::<RawDocument<RawCompanyFacts>>()
    /// #     .root("data")
    /// #     .build();
    /// let context = PrepareSecRequestContext::new(cik, repository);
    /// let state = PrepareSecRequest::new(input, context);
    ///
    /// let expected_result = false;
    ///
    /// let result = state.output_data().is_some();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(
        input: PrepareSecRequestInput<K>,
        context: PrepareSecRequestContext<K, R>,
    ) -> Self {
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
        PrepareSecRequestInput<K>,
        Option<PrepareSecRequestOutput<K>>,
        PrepareSecRequestContext<K, R>,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl<K: StoredAs, R: RawRepository<K::Source>> State for PrepareSecRequest<K, R> {
    /// Builds the [`SecRequest`] for the request input and stores it as output.
    ///
    /// # Errors
    ///
    /// Infallible in practice; the [`Result`] exists to satisfy the [`State`] contract.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let sec_request = SecRequest::new(self.input.request_input.clone());

        self.output = Some(PrepareSecRequestOutput::new(sec_request));

        Ok(())
    }
}

impl<K: StoredAs, R: RawRepository<K::Source>> SMState for PrepareSecRequest<K, R> {
    type InputData = PrepareSecRequestInput<K>;
    type OutputData = PrepareSecRequestOutput<K>;
    type Context = PrepareSecRequestContext<K, R>;

    /// Returns the human-readable name of this state, which its kind sets.
    fn state_name(&self) -> impl ToString {
        K::PREPARE_STATE_NAME
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

impl<K: StoredAs, R: RawRepository<K::Source>> fmt::Display for PrepareSecRequest<K, R> {
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
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use tokio;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::shared::notes_period::NotesPeriod;
    use crate::tests::fixtures::sample_write_repository::{
        FakeNotesRepository, FakeWriteRepository,
    };
    use crate::traits::state_machine::state::State;

    fn create_test_cik() -> Cik {
        Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid")
    }

    fn create_test_input() -> PrepareSecRequestInput<CompanyFacts> {
        PrepareSecRequestInput::new(create_test_cik())
    }

    fn create_test_context() -> PrepareSecRequestContext<CompanyFacts, FakeWriteRepository> {
        PrepareSecRequestContext::new(create_test_cik(), FakeWriteRepository::new())
    }

    fn create_test_state() -> PrepareCompanyFactsRequest<FakeWriteRepository> {
        PrepareSecRequest::new(create_test_input(), create_test_context())
    }

    #[test]
    fn should_return_name_of_prepare_state_when_in_prepare_state() {
        let prepare_state = create_test_state();
        let expected_result = String::from("Prepare SEC Request");
        let result = prepare_state.state_name().to_string();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_prepare_data_struct_as_input_data_when_in_initial_prepare_state() {
        let prepare_state = create_test_state();
        let expected_result = &create_test_input();
        let result = prepare_state.input_data();
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let prepare_state = create_test_state();
        let _result = prepare_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let prepare_state = create_test_state();
        let expected_result = false;
        let result = prepare_state.has_output_data_been_computed();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let prepare_state = create_test_state();
        let expected_result = &create_test_context();
        let result = prepare_state.context_data();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_new_prepare_state_with_provided_input_and_context() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let input = PrepareSecRequestInput::new(cik);
        let context = create_test_context();

        let expected_input = input.clone();
        let expected_context = context.clone();

        let result = PrepareSecRequest::new(input, context);

        assert_eq!(result.input_data(), &expected_input);
        assert_eq!(result.context_data(), &expected_context);
        assert!(result.output_data().is_none());
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_prepare_sec_request() {
        implements_auto_traits::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_prepare_sec_request() {
        implements_send::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_sync_for_prepare_sec_request() {
        implements_sync::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    #[test]
    const fn should_be_thread_safe_for_prepare_sec_request() {
        implements_send::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
        implements_sync::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_prepare_sec_request() {
        implements_sized::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_prepare_sec_request() {
        implements_hash::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_prepare_sec_request() {
        implements_partial_eq::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_prepare_sec_request() {
        implements_eq::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_prepare_sec_request() {
        implements_partial_ord::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_prepare_sec_request() {
        implements_ord::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_prepare_sec_request() {
        implements_debug::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_prepare_sec_request() {
        implements_clone::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_prepare_sec_request() {
        implements_unpin::<PrepareCompanyFactsRequest<FakeWriteRepository>>();
    }

    #[test]
    fn should_return_default_context_data_when_called_with_state_reference() {
        let prepare_state = &create_test_state();
        let ref_to_prepare_state = &create_test_state();

        let expected_result = prepare_state.context_data();
        let result = ref_to_prepare_state.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_prepare_state = &mut create_test_state();
        let expected_result = false;
        let result = ref_to_prepare_state.has_output_data_been_computed();
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_reference_state()
     {
        let ref_to_prepare_state = &create_test_state();
        let _result = ref_to_prepare_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_name_of_prepare_state_when_calling_reference_to_prepare_state() {
        let ref_to_prepare_state = &create_test_state();
        let expected_result = String::from("Prepare SEC Request");
        let result = ref_to_prepare_state.state_name().to_string();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_as_input_data_when_reference_prepare_state_in_initial_state()
     {
        let ref_to_prepare_state = &create_test_state();
        let expected_result = &create_test_input();
        let result = ref_to_prepare_state.input_data();
        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_change_input_data_when_computing_output_data() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let input = PrepareSecRequestInput::new(cik);
        let context = create_test_context();
        let mut prepare_state = PrepareSecRequest::new(input, context);

        let expected_result = &prepare_state.input_data().clone();

        prepare_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");
        let result = prepare_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_correct_output_data_when_computing_output_data() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let input = PrepareSecRequestInput::new(cik);
        let context = create_test_context();
        let mut prepare_state = PrepareSecRequest::new(input, context);

        let expected_result = "https://data.sec.gov/api/xbrl/companyfacts/CIK1234567890.json";

        prepare_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");
        let result = prepare_state
            .output_data()
            .unwrap()
            .request
            .inner
            .url()
            .as_str();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_true_when_output_data_has_been_computed() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let input = PrepareSecRequestInput::new(cik);
        let context = create_test_context();
        let mut prepare_state = PrepareSecRequest::new(input, context);

        let expected_result = true;

        prepare_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");
        let result = prepare_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_output_when_calling_sync_compute_outside_tokio_runtime() {
        let mut state = create_test_state();

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let mut state = create_test_state();

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_build_notes_file_request_when_computing_output_for_notes_file_kind() {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a published period, NotesPeriod creation should always succeed");
        let mut state = PrepareSecRequest::builder()
            .notes_file()
            .period(period)
            .repository(FakeNotesRepository::new())
            .build();

        let expected_result = Some(&PrepareSecRequestOutput::new(SecRequest::new(period)));

        state
            .compute_output_data_async()
            .await
            .expect("Request preparation should always succeed for a valid period");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    fn create_test_notes_state() -> PrepareNotesRequest<FakeNotesRepository> {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a published period, NotesPeriod creation should always succeed");
        PrepareSecRequest::builder()
            .notes_file()
            .period(period)
            .repository(FakeNotesRepository::new())
            .build()
    }

    #[test]
    fn should_return_prepare_notes_request_as_state_name_for_notes_file_kind() {
        let state = create_test_notes_state();

        let expected_result = "Prepare Notes Request";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_notes_file_url_when_computing_output_for_notes_file_kind() {
        let mut state = create_test_notes_state();

        let expected_result = "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2024q1_notes.zip";

        state
            .compute_output_data_async()
            .await
            .expect("Request preparation should always succeed for a valid period");
        let result = state
            .output_data()
            .expect("Given a computed state, the output should always be present")
            .request
            .inner
            .url()
            .as_str();

        assert_eq!(result, expected_result);
    }

    const fn implements_state<T: State>() {}
    #[test]
    const fn should_implement_state_for_prepare_notes_request_with_notes_repository() {
        implements_state::<PrepareNotesRequest<FakeNotesRepository>>();
    }
}
