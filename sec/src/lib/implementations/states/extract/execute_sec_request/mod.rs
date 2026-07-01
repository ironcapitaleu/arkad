//! # Execute SEC Request State
//!
//! Provides the [`ExecuteSecRequest`] state, the final step of the extract phase, which
//! sends the prepared request to the SEC API and captures the response.
//!
//! This is the one extract state that touches the network: it consumes the prepared client
//! and request, performs the HTTP call, and produces a
//! [`SecResponse`](crate::shared::response::implementations::sec_response::SecResponse) for the
//! transform phase. Isolating the network call here keeps the earlier states pure and testable.
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
//! let context = ExecuteSecRequestContext::new(cik);
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
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::ExecuteSecRequestContext;
pub use data::ExecuteSecRequestInput;
pub use data::ExecuteSecRequestOutput;

/// Final extract-phase state, sending the prepared request to the SEC API.
///
/// Consumes the prepared [`SecClient`](crate::shared::http_client::implementations::sec_client::SecClient)
/// and [`SecRequest`](crate::shared::request::implementations::sec_request::SecRequest), performs the
/// HTTP call, and stores the resulting
/// [`SecResponse`](crate::shared::response::implementations::sec_response::SecResponse). It is the only
/// extract state that performs I/O, which is why network failures surface here.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct ExecuteSecRequest {
    input: ExecuteSecRequestInput,
    context: ExecuteSecRequestContext,
    output: Option<ExecuteSecRequestOutput>,
}

impl ExecuteSecRequest {
    /// Creates a new state from its input and context, with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::execute_sec_request::*;
    /// use sec::shared::cik::Cik;
    /// use sec::shared::http_client::implementations::sec_client::SecClient;
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    ///
    /// let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
    /// let request = SecRequest::builder().all_company_facts().cik(cik.clone()).build();
    /// let input = ExecuteSecRequestInput::new(SecClient::default(), request);
    /// let context = ExecuteSecRequestContext::new(cik);
    ///
    /// let state = ExecuteSecRequest::new(input, context);
    /// ```
    #[must_use]
    pub const fn new(input: ExecuteSecRequestInput, context: ExecuteSecRequestContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    ///
    /// Used by transitions to move the SEC response into the transform phase without cloning.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        ExecuteSecRequestInput,
        Option<ExecuteSecRequestOutput>,
        ExecuteSecRequestContext,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl State for ExecuteSecRequest {
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

impl SMState for ExecuteSecRequest {
    type InputData = ExecuteSecRequestInput;
    type OutputData = ExecuteSecRequestOutput;
    type Context = ExecuteSecRequestContext;

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

impl fmt::Display for ExecuteSecRequest {
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

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::request::implementations::sec_request::SecRequest;

    const TEST_CIK: &str = "0001067983";

    fn create_test_cik() -> Cik {
        Cik::new(TEST_CIK).expect("Hardcoded CIK should be valid")
    }

    /// Creates a baseline `ExecuteSecRequest` state for use in tests.
    fn create_baseline_state() -> ExecuteSecRequest {
        let cik = create_test_cik();
        let client = SecClient::default();
        let request = SecRequest::builder()
            .all_company_facts()
            .cik(cik.clone())
            .build();
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::new(cik);
        ExecuteSecRequest::new(input, context)
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
        let context = ExecuteSecRequestContext::new(create_test_cik());

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
        let context = ExecuteSecRequestContext::new(create_test_cik());

        let expected_result = true;

        let state = ExecuteSecRequest::new(input, context);
        let result = state.output_data().is_none(); // We check if output data is None, since it should not be set before computation

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_trait() {
        implements_auto_traits::<ExecuteSecRequest>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_trait() {
        implements_send::<ExecuteSecRequest>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_trait() {
        implements_sync::<ExecuteSecRequest>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_trait() {
        implements_send::<ExecuteSecRequest>();
        implements_sync::<ExecuteSecRequest>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_trait() {
        implements_sized::<ExecuteSecRequest>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_trait() {
        implements_hash::<ExecuteSecRequest>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_trait() {
        implements_partial_eq::<ExecuteSecRequest>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_trait() {
        implements_eq::<ExecuteSecRequest>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_trait() {
        implements_partial_ord::<ExecuteSecRequest>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_trait() {
        implements_ord::<ExecuteSecRequest>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_trait() {
        implements_debug::<ExecuteSecRequest>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_trait() {
        implements_clone::<ExecuteSecRequest>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_trait() {
        implements_unpin::<ExecuteSecRequest>();
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
        let client = SecClient::default();
        let cik = Cik::new("0001067983").expect("Hardcoded CIK should be valid");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::new(create_test_cik());
        let mut execute_state = ExecuteSecRequest::new(input, context);

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
        let client = SecClient::default();
        let cik = Cik::new("0001067983").expect("Hardcoded CIK should be valid");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::new(create_test_cik());
        let mut execute_state = ExecuteSecRequest::new(input, context);

        execute_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");

        let result = execute_state.output_data();

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn should_return_true_when_output_data_has_been_computed() {
        let client = SecClient::default();
        let cik = Cik::new("0001067983").expect("Hardcoded CIK should be valid");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::new(create_test_cik());
        let mut execute_state = ExecuteSecRequest::new(input, context);

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
        let client = SecClient::default();
        let cik = Cik::new("0001067983").expect("Hardcoded CIK should be valid");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::new(create_test_cik());
        let mut execute_state = ExecuteSecRequest::new(input, context);

        let expected_result = true;
        let result = execute_state.compute_output_data_async().await.is_ok();

        assert_eq!(result, expected_result);
    }
}
