//! # Execute SEC Request State
//!
//! Provides the [`ExecuteSecRequest`] state, which sends the prepared request to the SEC API
//! and captures the response.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`ExecuteSecRequestContext`] carried alongside the state.
//! - [`data`]: The [`ExecuteSecRequestInput`] and [`ExecuteSecRequestOutput`] data types.
//!
//! ## Usage
//!
//! ```no_run
//! use sec::implementations::states::extract::execute_sec_request::*;
//! use sec::prelude::*;
//! use sec::shared::cik::Cik;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use sec::shared::request::implementations::sec_request::SecRequest;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
//! let request = SecRequest::builder().all_company_facts().cik(cik.clone()).build();
//! let input = ExecuteSecRequestInput::new(SecClient::default(), request);
//! # use sec::shared::raw_document::RawDocument;
//! # use sec::shared::raw_source::RawCompanyFacts;
//! # use storage::implementations::repository::filesystem::FilesystemRepository;
//! # let repository = FilesystemRepository::builder()
//! #     .writer()
//! #     .document_type::<RawDocument<RawCompanyFacts>>()
//! #     .root("data")
//! #     .build();
//! let context = ExecuteSecRequestContext::new(cik, repository);
//!
//! let mut state = ExecuteSecRequest::new(input, context);
//! state
//!     .compute_output_data_async()
//!     .await
//!     .expect("The live SEC request should succeed");
//!
//! let _response = state
//!     .output_data()
//!     .expect("Output is present after a successful computation")
//!     .response();
//! # }
//! ```
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract`]: Parent module describing the extraction flow.
//! - [`crate::shared::response::implementations::sec_response::SecResponse`]: The response type produced by this state.
//! - [`crate::traits::state_machine::state::State`]: The trait implemented by [`ExecuteSecRequest`].

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::error::state_machine::state::failed_request_execution::FailedRequestExecution;
use crate::shared::http_client::RespondsWith;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::raw_repository::RawRepository;
use crate::shared::request::implementations::sec_request::CompanyFacts;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::ExecuteSecRequestContext;
pub use data::ExecuteSecRequestInput;
pub use data::ExecuteSecRequestOutput;

/// Sends the prepared request to the SEC API and captures the response.
///
/// Consumes the prepared SEC client `C`, by default [`SecClient`], and the
/// [`SecRequest`](crate::shared::request::implementations::sec_request::SecRequest) of endpoint
/// `E`. It performs the HTTP call and stores the resulting [`SecResponse`]. The endpoint `E` sets
/// the body kind of the response.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
#[serde(bound(serialize = "R: Serialize, C: Serialize"))]
pub struct ExecuteSecRequest<E: RespondsWith, R, C = SecClient> {
    input: ExecuteSecRequestInput<E, C>,
    context: ExecuteSecRequestContext<E, R>,
    output: Option<ExecuteSecRequestOutput<E>>,
}

/// Alias of [`ExecuteSecRequest`] for the [`CompanyFacts`] kind.
///
/// It is the same type under a shorter name, not a separate state.
pub type ExecuteCompanyFactsRequest<R, C = SecClient> = ExecuteSecRequest<CompanyFacts, R, C>;

impl<E: RespondsWith, R, C> ExecuteSecRequest<E, R, C> {
    /// Creates a new [`ExecuteSecRequest`] state from its input and context, with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::execute_sec_request::*;
    /// use sec::shared::cik::Cik;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    /// use state_maschine::prelude::State as SMState;
    ///
    /// let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
    /// let request = SecRequest::builder().all_company_facts().cik(cik.clone()).build();
    /// let input = ExecuteSecRequestInput::new(SecClient::default(), request);
    /// # use sec::shared::raw_document::RawDocument;
    /// # use sec::shared::raw_source::RawCompanyFacts;
    /// # use storage::implementations::repository::filesystem::FilesystemRepository;
    /// # let repository = FilesystemRepository::builder()
    /// #     .writer()
    /// #     .document_type::<RawDocument<RawCompanyFacts>>()
    /// #     .root("data")
    /// #     .build();
    /// let context = ExecuteSecRequestContext::new(cik, repository);
    /// let state = ExecuteSecRequest::new(input, context);
    ///
    /// let expected_result = false;
    ///
    /// let result = state.output_data().is_some();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(
        input: ExecuteSecRequestInput<E, C>,
        context: ExecuteSecRequestContext<E, R>,
    ) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    #[must_use]
    #[expect(
        clippy::type_complexity,
        reason = "The tuple names the three parts of the state. An alias would hide them from the reader"
    )]
    pub fn into_parts(
        self,
    ) -> (
        ExecuteSecRequestInput<E, C>,
        Option<ExecuteSecRequestOutput<E>>,
        ExecuteSecRequestContext<E, R>,
    ) {
        (self.input, self.output, self.context)
    }

    /// Returns the state with `output` set. Tests use it to skip the network call.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_output(mut self, output: ExecuteSecRequestOutput<E>) -> Self {
        self.output = Some(output);
        self
    }
}

// Deviation: the output holds an `SecResponse<E::Body>`, and `FailedRequestExecution`
// wraps a `FailedSecRequest`. So the impls that compute the output pin the response and the error
// of `C`. Other impls bind `C: SecClient` only.
#[async_trait]
impl<E, R, C> State for ExecuteSecRequest<E, R, C>
where
    E: RespondsWith,
    R: RawRepository<E::Source>,
    C: SecClientTrait<Response<E::Body> = SecResponse<E::Body>, Error = FailedSecRequest>,
{
    /// Executes the prepared SEC request and stores the response as output.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::FailedRequestExecution`] when the HTTP request fails — for
    /// example on a network error, timeout, or non-success SEC API response.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let client = &self.input.sec_client;
        let request = &self.input.sec_request;

        let result = client.execute_sec_request(request.clone()).await;

        match result {
            Ok(response) => {
                self.output = Some(ExecuteSecRequestOutput::new(response));
                Ok(())
            }
            Err(e) => {
                let e: StateError =
                    FailedRequestExecution::new(self.state_name().to_string(), e).into();
                return Err(e);
            }
        }
    }
}

impl<E, R, C> SMState for ExecuteSecRequest<E, R, C>
where
    E: RespondsWith,
    R: RawRepository<E::Source>,
    C: SecClientTrait<Response<E::Body> = SecResponse<E::Body>, Error = FailedSecRequest>,
{
    type InputData = ExecuteSecRequestInput<E, C>;
    type OutputData = ExecuteSecRequestOutput<E>;
    type Context = ExecuteSecRequestContext<E, R>;

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

impl<E: RespondsWith, R: RawRepository<E::Source>, C: SecClientTrait> fmt::Display
    for ExecuteSecRequest<E, R, C>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            STATE_NAME,
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

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::implementations::sec_request::{NotesFile, SecRequest};
    use crate::shared::response::SecResponse as SecResponseTrait;
    use crate::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use crate::shared::status_code::StatusCode;
    use crate::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_write_repository::{
        FakeNotesRepository, FakeWriteRepository,
    };

    const TEST_CIK: &str = "0001067983";

    fn create_test_cik() -> Cik {
        Cik::new(TEST_CIK).expect("Hardcoded CIK should be valid")
    }

    /// Creates a baseline `ExecuteSecRequest` state for use in tests.
    fn create_baseline_state() -> ExecuteCompanyFactsRequest<FakeWriteRepository> {
        let cik = create_test_cik();
        let client = SecClient::default();
        let request = SecRequest::builder()
            .all_company_facts()
            .cik(cik.clone())
            .build();
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::new(cik, FakeWriteRepository::new());
        ExecuteSecRequest::new(input, context)
    }

    const FAKE_COMPANY_FACTS_BODY: &str = r#"{"cik":1067983,"facts":{}}"#;

    /// Creates an `ExecuteSecRequest` state whose fake client answers with `status` and a JSON body.
    fn create_fake_client_state(
        status: u16,
    ) -> ExecuteCompanyFactsRequest<FakeWriteRepository, AlwaysRespondingSecClient> {
        let cik = create_test_cik();
        let client = AlwaysRespondingSecClient::new(
            status,
            vec![("content-type", "application/json".to_string())],
            FAKE_COMPANY_FACTS_BODY.as_bytes().to_vec(),
        );
        let request = SecRequest::builder()
            .all_company_facts()
            .cik(cik.clone())
            .build();
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::new(cik, FakeWriteRepository::new());
        ExecuteSecRequest::new(input, context)
    }

    /// Creates an `ExecuteSecRequest<NotesFile, …>` state whose fake client answers with `status`
    /// and the ZIP fixture.
    fn create_notes_file_state(
        status: u16,
    ) -> ExecuteSecRequest<NotesFile, FakeNotesRepository, AlwaysRespondingSecClient> {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed");
        let client = AlwaysRespondingSecClient::new(
            status,
            vec![
                ("content-type", "application/zip".to_string()),
                ("content-length", SAMPLE_NOTES_ZIP.len().to_string()),
            ],
            SAMPLE_NOTES_ZIP.to_vec(),
        );
        let request = SecRequest::builder().notes_file().period(period).build();
        ExecuteSecRequest::new(
            ExecuteSecRequestInput::new(client, request),
            ExecuteSecRequestContext::new(period, FakeNotesRepository::new()),
        )
    }

    #[tokio::test]
    async fn should_output_the_zip_body_unchanged_when_the_fake_client_downloads_a_notes_file() {
        let mut state = create_notes_file_state(200);

        let expected_result = Some(SAMPLE_NOTES_ZIP);

        state
            .compute_output_data_async()
            .await
            .expect("Given status 200, a ZIP content type and a matching length, the download should always succeed");
        let result = state.output_data().map(|output| output.response().body());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_request_execution_when_the_fake_client_returns_status_404_for_a_notes_file()
     {
        let mut state = create_notes_file_state(404);

        let expected_result: Result<(), StateError> = Err(FailedRequestExecution::new(
            STATE_NAME,
            FailedSecRequest::from(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
                status_code: StatusCode::NotFound,
            })),
        )
        .into());

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_json_body_when_the_fake_client_returns_status_200() {
        let mut state = create_fake_client_state(200);

        let expected_result: Option<serde_json::Value> = Some(
            serde_json::from_str(FAKE_COMPANY_FACTS_BODY)
                .expect("Given a hardcoded valid JSON string, parsing should always succeed"),
        );

        state
            .compute_output_data_async()
            .await
            .expect("Given status 200 and a JSON body, the fake request should always succeed");
        let result = state
            .output_data()
            .map(|output| output.response().body().clone());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_request_execution_when_the_fake_client_returns_status_404() {
        let mut state = create_fake_client_state(404);

        let expected_result: Result<(), StateError> = Err(FailedRequestExecution::new(
            STATE_NAME,
            FailedSecRequest::from(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
                status_code: StatusCode::NotFound,
            })),
        )
        .into());

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_name_of_execute_state_when_in_execute_state() {
        let execute_state = create_baseline_state();

        let expected_result = String::from("Execute SEC Request");

        let result = execute_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_input_data_when_in_initial_execute_state() {
        let execute_state = create_baseline_state();
        let baseline = create_baseline_state();

        let expected_result = baseline.input_data();

        let result = execute_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "State should not have output data before computation")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let execute_state = create_baseline_state();
        let _result = execute_state
            .output_data()
            .expect("State should not have output data before computation");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let execute_state = create_baseline_state();

        let expected_result = false;

        let result = execute_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_store_provided_context_when_creating_new_execute_state() {
        let client = SecClient::default();
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should be valid");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::new(create_test_cik(), FakeWriteRepository::new());

        let expected_result = &context.clone();

        let state = ExecuteSecRequest::new(input, context);
        let result = state.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_have_no_output_data_when_creating_new_execute_state() {
        let client = SecClient::default();
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should be valid");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::new(create_test_cik(), FakeWriteRepository::new());

        let expected_result = true;

        let state = ExecuteSecRequest::new(input, context);
        let result = state.output_data().is_none(); // We check if output data is None, since it should not be set before computation

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_execute_sec_request() {
        implements_auto_traits::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_execute_sec_request() {
        implements_send::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_sync_for_execute_sec_request() {
        implements_sync::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    #[test]
    const fn should_be_thread_safe_for_execute_sec_request() {
        implements_send::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
        implements_sync::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_execute_sec_request() {
        implements_sized::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_execute_sec_request() {
        implements_hash::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_execute_sec_request() {
        implements_partial_eq::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_for_execute_sec_request() {
        implements_eq::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_for_execute_sec_request() {
        implements_partial_ord::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_for_execute_sec_request() {
        implements_ord::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_execute_sec_request() {
        implements_debug::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_execute_sec_request() {
        implements_clone::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_for_execute_sec_request() {
        implements_unpin::<ExecuteCompanyFactsRequest<FakeWriteRepository>>();
    }

    #[test]
    fn should_return_context_data_when_called_with_state_reference() {
        let execute_state = &create_baseline_state();
        let ref_to_execute_state = &create_baseline_state();

        let expected_result = execute_state.context_data();

        let result = ref_to_execute_state.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_execute_state = &mut create_baseline_state();

        let expected_result = false;

        let result = ref_to_execute_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_reference_state()
     {
        let ref_to_execute_state = &create_baseline_state();
        let _result = ref_to_execute_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_name_of_execute_state_when_calling_reference_to_execute_state() {
        let ref_to_execute_state = &create_baseline_state();

        let expected_result = String::from("Execute SEC Request");

        let result = ref_to_execute_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_input_data_when_reference_execute_state_in_initial_state() {
        let ref_to_execute_state = &create_baseline_state();
        let baseline = create_baseline_state();

        let expected_result = baseline.input_data();

        let result = ref_to_execute_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_change_input_data_when_computing_output_data() {
        let mut execute_state = create_fake_client_state(200);

        let expected_result = &execute_state.input_data().clone();

        execute_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");
        let result = execute_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_correct_output_data_when_computing_output_data() {
        let mut execute_state = create_fake_client_state(200);

        execute_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");

        let result = execute_state.output_data();

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn should_return_true_when_output_data_has_been_computed() {
        let mut execute_state = create_fake_client_state(200);

        let expected_result = true;

        execute_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");
        let result = execute_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_succeed_when_valid_input_is_provided() {
        let mut execute_state = create_fake_client_state(200);

        let expected_result = true;
        let result = execute_state.compute_output_data_async().await.is_ok();

        assert_eq!(result, expected_result);
    }
}
