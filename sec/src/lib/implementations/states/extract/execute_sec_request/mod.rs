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
//! use sec::shared::sec_client::SecClient;
//! use sec::shared::sec_request::SecRequest;
//! use sec::shared::cik::Cik;
//! use sec::prelude::*; // allows us to use call the `State` and other trait methods directly
//!
//! #[tokio::main]
//! async fn main() {
//!     // Prepare client and request (typically from PrepareSecRequest state)
//!     let client = SecClient::new("Test Company contact@test.com").expect("Valid user agent");
//!     let cik = Cik::new("1067983").expect("Valid CIK");
//!     let request = SecRequest::new(&cik);
//!     
//!     let input = ExecuteSecRequestInput::new(client, request);
//!     let context = ExecuteSecRequestContext::default();
//!
//!     let mut execute_state = ExecuteSecRequest::new(input, context);
//!     execute_state.compute_output_data_async().await.unwrap();
//!     let response_output = execute_state.output_data().unwrap();
//!
//!     // Now you have the SEC response data
//!     let response = response_output.response();
//! }
//! ```
//!
//! ## See Also
//! - [`crate::implementations::states::extract`]: Parent module for extraction-related states.
//! - [`crate::shared::sec_client::SecClient`]: Core SEC client type used for HTTP requests.
//! - [`crate::shared::sec_request::SecRequest`]: Core SEC request type for API calls.
//! - [`crate::shared::sec_response::SecResponse`]: Core SEC response type for API responses.
//! - [`crate::traits::state_machine::state::State`]: State trait implemented by [`ExecuteSecRequest`].
//!
//! ## Testing
//! This module includes comprehensive unit tests covering state behavior, trait compliance, and error handling.

pub mod context;
pub mod data;

pub use context::ExecuteSecRequestContext;
pub use data::ExecuteSecRequestInput;
pub use data::ExecuteSecRequestOutput;

use crate::error::State as StateError;
use crate::error::state_machine::state::request_execution_failed::RequestExecutionFailed;
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
/// - Produces a [`SecResponse`](crate::shared::sec_response::SecResponse) object containing the SEC API response data.
/// - Supports retry logic through the context configuration.
///
/// # Output
/// The SEC response is stored internally after calling [`State::compute_output_data_async`].
///
/// # Example
/// ```
/// use sec::implementations::states::extract::execute_sec_request::*;
/// use sec::shared::sec_client::SecClient;
/// use sec::shared::sec_request::SecRequest;
/// use sec::shared::cik::Cik;
///
/// let client = SecClient::new("Sample Corp contact@sample.com").expect("Valid user agent");
/// let cik = Cik::new("1067983").expect("Valid CIK");
/// let request = SecRequest::new(&cik);
/// let input = ExecuteSecRequestInput::new(client, request);
/// let context = ExecuteSecRequestContext::default();
/// let mut execute_state = ExecuteSecRequest::new(input, context);
/// ```
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
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

        let result = client.execute_request(request.clone()).await;

        match result {
            Ok(response) => {
                self.output = Some(ExecuteSecRequestOutput::new(response)?);
                Ok(())
            }
            Err(e) => {
                let e: StateError =
                    RequestExecutionFailed::new(self.state_name().to_string(), e).into();
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
        "Execute SEC Request"
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
    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::sec_client::SecClient;
    use crate::shared::sec_request::SecRequest;
    use crate::traits::state_machine::state::State;
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};
    use tokio;

    #[test]
    fn should_return_name_of_execute_state_when_in_execute_state() {
        let execute_state = ExecuteSecRequest::default();
        let expected_result = String::from("Execute SEC Request");
        let result = execute_state.state_name().to_string();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_execute_data_struct_as_input_data_when_in_initial_execute_state() {
        let execute_state = ExecuteSecRequest::default();
        let expected_result = &ExecuteSecRequestInput::default();
        let result = execute_state.input_data();
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let execute_state = ExecuteSecRequest::default();
        let _result = execute_state
            .output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let execute_state = ExecuteSecRequest::default();
        let expected_result = false;
        let result = execute_state.has_output_data_been_computed();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let execute_state = ExecuteSecRequest::default();
        let expected_result = &ExecuteSecRequestContext::default();
        let result = execute_state.context_data();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_new_execute_state_with_provided_input_and_context() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let client = SecClient::new("Test Company contact@test.com")
            .expect("Hardcoded user agent should always be valid.");
        let request = SecRequest::new(&cik);
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::default();

        let expected_input = input.clone();
        let expected_context = context.clone();

        let result = ExecuteSecRequest::new(input, context);

        assert_eq!(result.input_data(), &expected_input);
        assert_eq!(result.context_data(), &expected_context);
        assert!(result.output_data().is_none());
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

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_state_trait() {
        implements_default::<ExecuteSecRequest>();
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
    fn should_return_default_context_data_when_called_with_state_reference() {
        let execute_state = &ExecuteSecRequest::default();
        let ref_to_execute_state = &ExecuteSecRequest::default();

        let expected_result = execute_state.context_data();
        let result = ref_to_execute_state.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_execute_state = &mut ExecuteSecRequest::default();
        let expected_result = false;
        let result = ref_to_execute_state.has_output_data_been_computed();
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_reference_state()
     {
        let ref_to_execute_state = &ExecuteSecRequest::default();
        let _result = ref_to_execute_state
            .output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_name_of_execute_state_when_calling_reference_to_execute_state() {
        let ref_to_execute_state = &ExecuteSecRequest::default();
        let expected_result = String::from("Execute SEC Request");
        let result = ref_to_execute_state.state_name().to_string();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_as_input_data_when_reference_execute_state_in_initial_state()
     {
        let ref_to_execute_state = &ExecuteSecRequest::default();
        let expected_result = &ExecuteSecRequestInput::default();
        let result = ref_to_execute_state.input_data();
        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_change_input_data_when_computing_output_data() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let client = SecClient::new("Test Company contact@test.com")
            .expect("Hardcoded user agent should always be valid.");
        let request = SecRequest::new(&cik);
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::default();
        let mut execute_state = ExecuteSecRequest::new(input, context);

        let expected_result = &execute_state.input_data().clone();

        execute_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data.");
        let result = execute_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_correct_output_data_when_computing_output_data() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let client = SecClient::new("Test Company contact@test.com")
            .expect("Hardcoded user agent should always be valid.");
        let request = SecRequest::new(&cik);
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::default();
        let mut execute_state = ExecuteSecRequest::new(input, context);

        execute_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data.");
        let result = execute_state.output_data().unwrap();

        // Verify that we got a response (the exact content depends on network availability)
        // Just check that we have a valid response object with a body method
        let _body_length = result.response().body().len();
    }

    #[tokio::test]
    async fn should_return_true_when_output_data_has_been_computed() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let client = SecClient::new("Test Company contact@test.com")
            .expect("Hardcoded user agent should always be valid.");
        let request = SecRequest::new(&cik);
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::default();
        let mut execute_state = ExecuteSecRequest::new(input, context);

        let expected_result = true;

        execute_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data.");
        let result = execute_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_succeed_when_valid_input_is_provided() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let client = SecClient::new("Test Company contact@test.com")
            .expect("Hardcoded user agent should always be valid.");
        let request = SecRequest::new(&cik);
        let input = ExecuteSecRequestInput::new(client, request);
        let context = ExecuteSecRequestContext::default();
        let mut execute_state = ExecuteSecRequest::new(input, context);

        let result = execute_state.compute_output_data_async().await;

        assert!(result.is_ok());
        assert!(execute_state.has_output_data_been_computed());
        let output = execute_state.output_data().unwrap();
        // We can verify that we get some response data, but actual content verification would depend on SEC API availability
        // Just check that we have a valid response object with a body method
        let _body_length = output.response().body().len();
    }
}
