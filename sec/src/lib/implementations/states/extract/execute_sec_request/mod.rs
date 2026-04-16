//! # SEC Request Execution State
//!
//! This module provides the [`ExecuteSecRequest`] state and related types for executing HTTP requests to SEC API endpoints as part of the SEC filings extraction workflow.
//!
//! ## Overview
//! The [`ExecuteSecRequest`] state is responsible for executing HTTP requests using a prepared SEC client and request object. It takes the configured client and request as input and produces a response containing the SEC data.
//!
//! ## Components
//! - [`context`]: Defines the context and updater types for the request execution process, allowing stateful tracking of execution-related context.
//! - [`data`]: Contains input and output data structures for the execution state, including updaters and builders for ergonomic data manipulation.
//! - [`ExecuteSecRequestContext`]: Context data type for the state.
//! - [`ExecuteSecRequestInput`]: Input data type holding the prepared SEC client and request.
//! - [`ExecuteSecRequestOutput`]: Output data type containing the SEC response.
//!
//! ## Usage
//! This state is typically used in the extract phase of the SEC state machine ETL pipeline, after request preparation and before response processing. It is designed to be composed with other states for robust and testable SEC filings processing workflows.
//!
//! ## Example
//! ```rust
//! use tokio;
//!
//! use sec::implementations::states::extract::execute_sec_request::*;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use sec::shared::request::implementations::sec_request::SecRequest;
//! use sec::shared::cik::Cik;
//! use sec::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = SecClient::default();
//!     let cik = Cik::new("1067983").expect("Hardcoded CIK string should be valid format");
//!     let request = SecRequest::builder()
//!         .all_company_facts()
//!         .cik(cik.clone())
//!         .build();
//!
//!     let input = ExecuteSecRequestInput::new(client, request);
//!     let context = ExecuteSecRequestContext::new(cik);
//!
//!     let mut execute_state = ExecuteSecRequest::new(input, context);
//!     execute_state.compute_output_data_async().await.unwrap();
//!     let response_output = execute_state.output_data().unwrap();
//!
//!     let response = response_output.response();
//! }
//! ```
//!
//! ## See Also
//! - [`crate::implementations::states::extract`]: Parent module for extraction-related states.
//! - [`crate::shared::http_client::implementations::sec_client::SecClient`]: SEC client type used for HTTP requests.
//! - [`crate::shared::request::implementations::sec_request::SecRequest`]: SEC request type for API calls.
//! - [`crate::traits::state_machine::state::State`]: State trait implemented by [`ExecuteSecRequest`].
//!
//! ## Testing
//! This module includes comprehensive unit tests covering state behavior, trait compliance, and error handling.

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::ExecuteSecRequestContext;
pub use data::ExecuteSecRequestInput;
pub use data::ExecuteSecRequestOutput;

use crate::error::State as StateError;
use crate::error::state_machine::state::failed_request_execution::FailedRequestExecution;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::traits::state_machine::state::State;

use std::fmt;

use async_trait::async_trait;
use state_maschine::prelude::State as SMState;

/// State that executes HTTP requests to SEC API endpoints.
///
/// This state takes a prepared SEC client and request object as input, executes the HTTP request
/// to the SEC API, and produces a response containing the requested SEC data. It handles request
/// execution and error handling for network-related failures.
///
/// # Behavior
/// - Executes HTTP requests using the provided SEC client and request configuration.
/// - Handles network errors and converts them to appropriate state errors.
/// - Produces a [`SecResponse`](crate::shared::response::implementations::sec_response::SecResponse) object containing the SEC API response data.
/// - Supports retry logic through the context configuration.
///
/// # Output
/// The SEC response is stored internally after calling [`State::compute_output_data_async`].
///
/// # Example
/// ```
/// use sec::implementations::states::extract::execute_sec_request::*;
/// use sec::shared::http_client::implementations::sec_client::SecClient;
/// use sec::shared::request::implementations::sec_request::SecRequest;
/// use sec::shared::cik::Cik;
///
/// let client = SecClient::default();
/// let cik = Cik::new("1067983").expect("Hardcoded CIK string should be valid format");
/// let request = SecRequest::builder()
///     .all_company_facts()
///     .cik(cik.clone())
///     .build();
/// let input = ExecuteSecRequestInput::new(client, request);
/// let context = ExecuteSecRequestContext::new(cik);
/// let mut execute_state = ExecuteSecRequest::new(input, context);
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, serde::Serialize)]
pub struct ExecuteSecRequest {
    input: ExecuteSecRequestInput,
    context: ExecuteSecRequestContext,
    output: Option<ExecuteSecRequestOutput>,
}

impl ExecuteSecRequest {
    /// Creates a new [`ExecuteSecRequest`] state with the provided input and context.
    ///
    /// # Arguments
    ///
    /// * `input` - The [`ExecuteSecRequestInput`] containing the prepared SEC client and request.
    /// * `context` - The [`ExecuteSecRequestContext`] for the execution process.
    ///
    /// # Returns
    ///
    /// Returns a new [`ExecuteSecRequest`] state ready for computation.
    #[must_use]
    pub const fn new(input: ExecuteSecRequestInput, context: ExecuteSecRequestContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }
}

#[async_trait]
impl State for ExecuteSecRequest {
    /// Computes the output data by executing the SEC HTTP request.
    ///
    /// This method takes the prepared SEC client and request from the input data,
    /// executes the HTTP request asynchronously, and stores the response in the output data.
    ///
    /// # Errors
    ///
    /// Returns a [`StateError`] if:
    /// - The HTTP request fails due to network issues, timeouts, or invalid responses
    /// - The SEC API returns an error response
    /// - The response cannot be processed or stored
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the request is successfully executed and the response is stored,
    /// or `Err(StateError)` if execution fails.
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

    /// Executes the SEC HTTP request and processes the response.
    /// Does nothing here, as the output data is computed in `compute_output_data_async()` of the `sec`'s `State`-implementation.
    fn compute_output_data(&mut self) {
        // No action needed here, as the output data is computed in `compute_output_data_async()` of the `sec`'s 'State'-implementation
        // This function is just a placeholder to satisfy the State trait.
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
    use crate::traits::state_machine::state::State;

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
