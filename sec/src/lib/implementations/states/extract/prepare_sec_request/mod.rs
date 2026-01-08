//! # SEC Request Preparation State
//!
//! This module provides the [`PrepareSecRequest`] state and related types for preparing HTTP clients and requests for SEC API interactions as part of the SEC filings extraction workflow.
//!
//! ## Overview
//! The [`PrepareSecRequest`] state is responsible for creating and configuring the necessary HTTP infrastructure to interact with SEC API endpoints. It takes a validated CIK and user agent string as input and produces a configured HTTP client and request object ready for SEC API calls.
//!
//! ## Components
//! - [`context`]: Defines the context and updater types for the request preparation process, allowing stateful tracking of preparation-related context.
//! - [`data`]: Contains input and output data structures for the preparation state, including updaters and builders for ergonomic data manipulation.
//! - [`PrepareSecRequestContext`]: Context data type for the state.
//! - [`PrepareSecRequestInput`]: Input data type holding the validated CIK and user agent string.
//! - [`PrepareSecRequestOutput`]: Output data type containing the prepared SEC client and request.
//!
//! ## Usage
//! This state is typically used in the extract phase of the SEC state machine ETL pipeline, after CIK validation and before making actual HTTP requests to SEC endpoints. It is designed to be composed with other states for robust and testable SEC filings processing workflows.
//!
//! ## Example
//! ```rust
//! use tokio;
//!
//! use sec::implementations::states::extract::prepare_sec_request::*;
//! use sec::shared::cik::Cik;
//! use sec::prelude::*; // allows us to use call the `State` and other trait methods directly
//!
//! #[tokio::main]
//! async fn main() {
//!     let cik = Cik::new("1067983").expect("Hardcoded CIK string should be valid format");
//!     let user_agent = "Test Company contact@test.com".to_string();
//!     let input = PrepareSecRequestInput::new(cik, user_agent);
//!     let context = PrepareSecRequestContext::default();
//!
//!     let mut prepare_state = PrepareSecRequest::new(input, context);
//!     prepare_state.compute_output_data_async().await.unwrap();
//!     let prepared_output = prepare_state.output_data().unwrap();
//!
//!     // Now you have a client and request ready for SEC API calls
//!     let client = prepared_output.client();
//!     let request = prepared_output.request();
//! }
//! ```
//!
//! ## See Also
//! - [`crate::implementations::states::extract`]: Parent module for extraction-related states.
//! - [`crate::shared::sec_client::SecClient`]: Core SEC client type used for HTTP requests.
//! - [`crate::shared::sec_request::SecRequest`]: Core SEC request type for API calls.
//! - [`crate::traits::state_machine::state::State`]: State trait implemented by [`PrepareSecRequest`].
//!
//! ## Testing
//! This module includes comprehensive unit tests covering state behavior, trait compliance, and error handling.

use std::fmt;

use async_trait::async_trait;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::error::state_machine::state::client_creation_failed::ClientCreationFailed;
use crate::shared::sec_client::SecClient;
use crate::shared::sec_request::SecRequest;
use crate::traits::state_machine::state::State;

pub mod context;
pub mod data;

pub use context::PrepareSecRequestContext;
pub use data::PrepareSecRequestInput;
pub use data::PrepareSecRequestOutput;

/// State that prepares HTTP client and request objects for SEC API interactions.
///
/// The state takes a validated CIK and user agent string as input, creates a configured
/// HTTP client with proper user agent headers, and constructs a request object targeting
/// the appropriate SEC API endpoint for the given CIK.
///
/// # Behavior
/// - Validates the user agent string format for SEC compliance.
/// - Creates an HTTP client configured with the validated user agent.
/// - Constructs a request object targeting the SEC submissions endpoint for the given CIK.
/// - Produces configured [`SecClient`] and [`SecRequest`] objects ready for API calls.
///
/// # Output
/// The prepared client and request are stored internally after calling [`State::compute_output_data_async`].
///
/// # Example
/// ```
/// use sec::implementations::states::extract::prepare_sec_request::*;
/// use sec::shared::cik::Cik;
///
/// let cik = Cik::new("1067983").expect("Hardcoded CIK string should be valid format");
/// let user_agent = "Sample Corp contact@sample.com".to_string();
/// let input = PrepareSecRequestInput::new(cik, user_agent);
/// let context = PrepareSecRequestContext::default();
/// let mut prepare_state = PrepareSecRequest::new(input, context);
/// ```
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequest {
    input: PrepareSecRequestInput,
    context: PrepareSecRequestContext,
    output: Option<PrepareSecRequestOutput>,
}

impl PrepareSecRequest {
    /// Creates a new [`PrepareSecRequest`] state with the provided input and context.
    ///
    /// # Arguments
    ///
    /// * `input` - The [`PrepareSecRequestInput`] containing the validated [`Cik`](crate::shared::cik::Cik) and user agent string.
    /// * `context` - The [`PrepareSecRequestContext`] for the preparation process.
    ///
    /// # Returns
    ///
    /// Returns a new [`PrepareSecRequest`] state ready for computation.
    #[must_use]
    pub const fn new(input: PrepareSecRequestInput, context: PrepareSecRequestContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }
}

#[async_trait]
impl State for PrepareSecRequest {
    /// Computes the output data by creating SEC client and request objects.
    ///
    /// This method validates the user agent format, creates an HTTP client configured
    /// with the user agent, and constructs a request object for the given CIK.
    ///
    /// # Errors
    ///
    /// Returns a [`StateError`] if:
    /// - The user agent string doesn't meet SEC format requirements
    /// - The HTTP client cannot be created due to configuration issues
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the client and request are successfully created and stored,
    /// or `Err(StateError)` if preparation fails.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let sec_client = SecClient::new(&self.input.user_agent);
        let sec_request = SecRequest::new(&self.input.validated_cik);

        match sec_client {
            Ok(client) => {
                self.output = Some(PrepareSecRequestOutput::new(client, sec_request)?);
                Ok(())
            }
            Err(e) => {
                let e: StateError =
                    ClientCreationFailed::new(self.state_name().to_string(), e).into();
                return Err(e);
            }
        }
    }
}

impl SMState for PrepareSecRequest {
    type InputData = PrepareSecRequestInput;
    type OutputData = PrepareSecRequestOutput;
    type Context = PrepareSecRequestContext;

    /// Returns the human-readable name of this state.
    fn state_name(&self) -> impl ToString {
        "Prepare SEC Request"
    }

    /// Prepares SEC client and request objects for API interactions.
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

impl fmt::Display for PrepareSecRequest {
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
    use crate::traits::state_machine::state::State;
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};
    use tokio;

    #[test]
    fn should_return_name_of_prepare_state_when_in_prepare_state() {
        let prepare_state = PrepareSecRequest::default();
        let expected_result = String::from("Prepare SEC Request");
        let result = prepare_state.state_name().to_string();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_prepare_data_struct_as_input_data_when_in_initial_prepare_state() {
        let prepare_state = PrepareSecRequest::default();
        let expected_result = &PrepareSecRequestInput::default();
        let result = prepare_state.input_data();
        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let prepare_state = PrepareSecRequest::default();
        let _result = prepare_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let prepare_state = PrepareSecRequest::default();
        let expected_result = false;
        let result = prepare_state.has_output_data_been_computed();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let prepare_state = PrepareSecRequest::default();
        let expected_result = &PrepareSecRequestContext::default();
        let result = prepare_state.context_data();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_new_prepare_state_with_provided_input_and_context() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let user_agent = "Test Company contact@test.com".to_string();
        let input = PrepareSecRequestInput::new(cik.clone(), user_agent.clone());
        let context = PrepareSecRequestContext::default();

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
    const fn should_still_implement_auto_traits_when_implementing_state_trait() {
        implements_auto_traits::<PrepareSecRequest>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_trait() {
        implements_send::<PrepareSecRequest>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_trait() {
        implements_sync::<PrepareSecRequest>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_trait() {
        implements_send::<PrepareSecRequest>();
        implements_sync::<PrepareSecRequest>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_trait() {
        implements_sized::<PrepareSecRequest>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_trait() {
        implements_hash::<PrepareSecRequest>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_trait() {
        implements_partial_eq::<PrepareSecRequest>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_trait() {
        implements_eq::<PrepareSecRequest>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_trait() {
        implements_partial_ord::<PrepareSecRequest>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_trait() {
        implements_ord::<PrepareSecRequest>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_state_trait() {
        implements_default::<PrepareSecRequest>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_trait() {
        implements_debug::<PrepareSecRequest>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_trait() {
        implements_clone::<PrepareSecRequest>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_trait() {
        implements_unpin::<PrepareSecRequest>();
    }

    #[test]
    fn should_return_default_context_data_when_called_with_state_reference() {
        let prepare_state = &PrepareSecRequest::default();
        let ref_to_prepare_state = &PrepareSecRequest::default();

        let expected_result = prepare_state.context_data();
        let result = ref_to_prepare_state.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_prepare_state = &mut PrepareSecRequest::default();
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
        let ref_to_prepare_state = &PrepareSecRequest::default();
        let _result = ref_to_prepare_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_name_of_prepare_state_when_calling_reference_to_prepare_state() {
        let ref_to_prepare_state = &PrepareSecRequest::default();
        let expected_result = String::from("Prepare SEC Request");
        let result = ref_to_prepare_state.state_name().to_string();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_as_input_data_when_reference_prepare_state_in_initial_state()
     {
        let ref_to_prepare_state = &PrepareSecRequest::default();
        let expected_result = &PrepareSecRequestInput::default();
        let result = ref_to_prepare_state.input_data();
        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_change_input_data_when_computing_output_data() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let user_agent = "Test Company contact@test.com".to_string();
        let input = PrepareSecRequestInput::new(cik, user_agent);
        let context = PrepareSecRequestContext::default();
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
        let user_agent = "Test Company contact@test.com".to_string();
        let input = PrepareSecRequestInput::new(cik, user_agent);
        let context = PrepareSecRequestContext::default();
        let mut prepare_state = PrepareSecRequest::new(input, context);

        prepare_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");
        let result = prepare_state.output_data().unwrap();

        assert!(!result.client().id().is_empty());
        assert!(
            result
                .request()
                .inner
                .url()
                .as_str()
                .contains("data.sec.gov")
        );
    }

    #[tokio::test]
    async fn should_return_true_when_output_data_has_been_computed() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let user_agent = "Test Company contact@test.com".to_string();
        let input = PrepareSecRequestInput::new(cik, user_agent);
        let context = PrepareSecRequestContext::default();
        let mut prepare_state = PrepareSecRequest::new(input, context);

        let expected_result = true;

        prepare_state
            .compute_output_data_async()
            .await
            .expect("Valid state should always compute output data");
        let result = prepare_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_when_user_agent_is_invalid() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let invalid_user_agent = "Invalid User Agent".to_string(); // Missing email
        let input = PrepareSecRequestInput::new(cik, invalid_user_agent);
        let context = PrepareSecRequestContext::default();
        let mut prepare_state = PrepareSecRequest::new(input, context);

        let result = prepare_state.compute_output_data_async().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_succeed_when_valid_input_is_provided() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let user_agent = "Test Company contact@test.com".to_string();
        let input = PrepareSecRequestInput::new(cik, user_agent);
        let context = PrepareSecRequestContext::default();
        let mut prepare_state = PrepareSecRequest::new(input, context);

        let result = prepare_state.compute_output_data_async().await;

        assert!(result.is_ok());
        assert!(prepare_state.has_output_data_been_computed());
        let output = prepare_state.output_data().unwrap();
        assert!(!output.client().id().is_empty());
        assert!(output.request().inner.url().as_str().contains("1234567890"));
    }
}
