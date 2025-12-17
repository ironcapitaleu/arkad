//! # SEC Response Validation State
//!
//! This module provides the [`ValidateSecResponse`] state and related types for validating HTTP responses from SEC API endpoints as part of the SEC filings extraction workflow.
//!
//! ## Overview
//! The [`ValidateSecResponse`] state is responsible for validating SEC HTTP responses and extracting valid JSON data. It takes a raw SEC response as input and produces a validated JSON response after checking status codes, content types, and JSON structure.
//!
//! ## Components
//! - [`context`]: Defines the context data and updater types for the response validation process, allowing stateful tracking of validation-related context.
//! - [`data`]: Contains input and output data structures for the validation state, including updaters and builders for ergonomic data manipulation.
//! - [`ValidateSecResponseContext`]: Context data type for the state.
//! - [`ValidateSecResponseInput`]: Input data type holding the raw SEC response.
//! - [`ValidateSecResponseOutput`]: Output data type containing the validated JSON response.
//!
//! ## Usage
//! This state is typically used in the extract phase of the SEC state machine ETL pipeline, after request execution and before data extraction. It is designed to be composed with other states for robust and testable SEC filings processing workflows.
//!
//! ## Example
//! ```rust
//! use tokio;
//!
//! use sec::implementations::states::extract::validate_sec_response::*;
//! use sec::shared::sec_response::SecResponse;
//! use sec::shared::cik::Cik;
//! use sec::prelude::*; // allows us to use call the `State` and other trait methods directly
//!
//! #[tokio::main]
//! async fn main() {
//!     // Prepare response (typically from ExecuteSecRequest state)
//!     let sec_response = SecResponse::default();
//!     
//!     let cik = Cik::new("1067983").expect("Valid CIK");
//!     let input = ValidateSecResponseInput::new(sec_response);
//!     let context = ValidateSecResponseContext::new(cik);
//!
//!     let mut validate_state = ValidateSecResponse::new(input, context);
//!     // validate_state.compute_output_data_async().await.unwrap();
//!     // let json_output = validate_state.get_output_data().unwrap();
//!
//!     // Now you have validated JSON data
//!     // let json_body = json_output.validated_sec_response();
//! }
//! ```
//!
//! ## See Also
//! - [`crate::implementations::states::extract`]: Parent module for extraction-related states.
//! - [`crate::shared::sec_response::SecResponse`]: Core SEC response type for API responses.
//! - [`crate::shared::json_response::JsonResponse`]: Core JSON response type for validated data.
//! - [`crate::traits::state_machine::state::State`]: State trait implemented by [`ValidateSecResponse`].
//!
//! ## Testing
//! This module includes comprehensive unit tests covering state behavior, trait compliance, and error handling.

use std::fmt;

use async_trait::async_trait;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::error::state_machine::state::InvalidSecResponse;
use crate::shared::json_response::JsonResponse;
use crate::traits::error::FromDomainError;
use crate::traits::state_machine::state::State;

pub mod context;
pub mod data;

pub use context::ValidateSecResponseContext;
pub use data::ValidateSecResponseInput;
pub use data::ValidateSecResponseOutput;

/// State that validates HTTP responses from SEC API endpoints.
///
/// This state takes a raw SEC response as input, validates its structure, status code,
/// content type, and JSON format, and produces a validated JSON response object. It handles
/// validation errors and converts them to appropriate state errors.
///
/// # Behavior
/// - Validates HTTP status codes (expects 2xx range).
/// - Verifies content type is JSON (`application/json`).
/// - Checks that the response body is not empty.
/// - Parses and validates JSON structure.
/// - Produces a [`JsonResponse`] object containing validated JSON data.
///
/// # Output
/// The validated JSON response is stored internally after calling [`State::compute_output_data_async`].
///
/// # Example
/// ```
/// use sec::implementations::states::extract::validate_sec_response::*;
/// use sec::shared::sec_response::SecResponse;
/// use sec::shared::cik::Cik;
///
/// let sec_response = SecResponse::default();
/// let cik = Cik::new("1067983").expect("Valid CIK");
/// let input = ValidateSecResponseInput::new(sec_response);
/// let context = ValidateSecResponseContext::new(cik);
/// let mut validate_state = ValidateSecResponse::new(input, context);
/// ```
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateSecResponse {
    input: ValidateSecResponseInput,
    context: ValidateSecResponseContext,
    output: Option<ValidateSecResponseOutput>,
}

impl ValidateSecResponse {
    /// Creates a new [`ValidateSecResponse`] state with the provided input and context data.
    ///
    /// # Arguments
    ///
    /// * `input` - The [`ValidateSecResponseInput`] containing the raw SEC response.
    /// * `context` - The [`ValidateSecResponseContext`] for the validation process.
    ///
    /// # Returns
    ///
    /// Returns a new [`ValidateSecResponse`] state ready for computation.
    #[must_use]
    pub const fn new(
        input: ValidateSecResponseInput,
        context: ValidateSecResponseContext,
    ) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }
}

#[async_trait]
impl State for ValidateSecResponse {
    /// Computes the output data by validating the SEC HTTP response.
    ///
    /// This method takes the raw SEC response from the input data, validates its structure,
    /// status code, content type, and JSON format, then stores the validated JSON response
    /// in the output data.
    ///
    /// # Errors
    ///
    /// Returns a [`StateError`] if:
    /// - The response status code is not successful (not in 2xx range)
    /// - The response body is empty
    /// - The content type is not JSON (`application/json`)
    /// - The response body contains invalid JSON structure
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the response is successfully validated and stored,
    /// or `Err(StateError)` if validation fails.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let validated_sec_response = JsonResponse::from_sec_response(self.input.sec_response());

        match validated_sec_response {
            Ok(validated_response) => {
                self.output = Some(ValidateSecResponseOutput {
                    validated_sec_response: validated_response,
                });
            }
            Err(e) => {
                let e: StateError =
                    InvalidSecResponse::from_domain_error(self.get_state_name().to_string(), e)
                        .into();
                return Err(e);
            }
        }

        Ok(())
    }
}

impl SMState for ValidateSecResponse {
    type InputData = ValidateSecResponseInput;
    type OutputData = ValidateSecResponseOutput;
    type Context = ValidateSecResponseContext;

    fn get_state_name(&self) -> impl ToString {
        "Validate SEC Response"
    }

    fn compute_output_data(&mut self) {}

    fn get_context_data(&self) -> &Self::Context {
        &self.context
    }

    fn get_input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn get_output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl fmt::Display for ValidateSecResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.get_state_name().to_string(),
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
    use reqwest::StatusCode;
    use tokio;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::sec_response::{ContentType, SecResponse};
    use crate::traits::state_machine::state::State;

    #[test]
    fn should_return_name_of_validate_state_when_in_validate_state() {
        let validate_state = ValidateSecResponse::default();

        let expected_result = String::from("Validate SEC Response");

        let result = validate_state.get_state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_validate_data_struct_as_input_data_when_in_initial_validate_state() {
        let validate_state = ValidateSecResponse::default();

        let expected_result = &ValidateSecResponseInput::default();

        let result = validate_state.get_input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let validate_state = ValidateSecResponse::default();

        let _result = validate_state
            .get_output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let validate_state = ValidateSecResponse::default();

        let expected_result = false;

        let result = validate_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let validate_state = ValidateSecResponse::default();

        let expected_result = &ValidateSecResponseContext::default();

        let result = validate_state.get_context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_new_validate_state_with_provided_input_and_context() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let sec_response = SecResponse::default();
        let input = ValidateSecResponseInput::new(sec_response.clone());
        let context = ValidateSecResponseContext::new(cik.clone());

        let expected_result = ValidateSecResponse {
            input: ValidateSecResponseInput::new(sec_response),
            context: ValidateSecResponseContext::new(cik),
            output: None,
        };

        let result = ValidateSecResponse::new(input, context);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_correct_output_data_when_computing_output_data() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let sec_response = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK1234567890.json",
            )
            .expect("Hardcoded URL should always be valid."),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"test\": \"data\"}"),
        };
        let input = ValidateSecResponseInput::new(sec_response);
        let context = ValidateSecResponseContext::new(cik);
        let mut validate_state = ValidateSecResponse::new(input, context);

        let expected_result = true;

        validate_state
            .compute_output_data_async()
            .await
            .expect("Should succeed with valid response.");
        let result = validate_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_true_when_output_data_has_been_computed() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let sec_response = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK1234567890.json",
            )
            .expect("Hardcoded URL should always be valid."),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"valid\": true}"),
        };
        let input = ValidateSecResponseInput::new(sec_response);
        let context = ValidateSecResponseContext::new(cik);
        let mut validate_state = ValidateSecResponse::new(input, context);

        validate_state
            .compute_output_data_async()
            .await
            .expect("Computation should succeed");
        let result = validate_state.has_output_data_been_computed();

        assert!(result);
    }

    #[tokio::test]
    async fn should_fail_when_response_status_code_is_not_success() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let sec_response = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK1234567890.json",
            )
            .expect("Hardcoded URL should always be valid."),
            status: StatusCode::BAD_REQUEST,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"error\": \"invalid\"}"),
        };
        let input = ValidateSecResponseInput::new(sec_response);
        let context = ValidateSecResponseContext::new(cik);
        let mut validate_state = ValidateSecResponse::new(input, context);

        let result = validate_state.compute_output_data_async().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_not_change_input_data_when_computing_output_data() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let sec_response = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK1234567890.json",
            )
            .expect("Hardcoded URL should always be valid."),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"data\": [1,2,3]}"),
        };
        let input = ValidateSecResponseInput::new(sec_response.clone());
        let context = ValidateSecResponseContext::new(cik);
        let mut validate_state = ValidateSecResponse::new(input, context);

        let expected_result = ValidateSecResponseInput::new(sec_response);

        validate_state
            .compute_output_data_async()
            .await
            .expect("Should succeed");
        let result = validate_state.get_input_data();

        assert_eq!(result, &expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_trait() {
        implements_auto_traits::<ValidateSecResponse>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_trait() {
        implements_send::<ValidateSecResponse>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_trait() {
        implements_sync::<ValidateSecResponse>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_trait() {
        implements_send::<ValidateSecResponse>();
        implements_sync::<ValidateSecResponse>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_trait() {
        implements_sized::<ValidateSecResponse>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_trait() {
        implements_hash::<ValidateSecResponse>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_trait() {
        implements_partial_eq::<ValidateSecResponse>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_trait() {
        implements_eq::<ValidateSecResponse>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_trait() {
        implements_partial_ord::<ValidateSecResponse>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_trait() {
        implements_ord::<ValidateSecResponse>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_state_trait() {
        implements_default::<ValidateSecResponse>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_trait() {
        implements_debug::<ValidateSecResponse>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_trait() {
        implements_clone::<ValidateSecResponse>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_trait() {
        implements_unpin::<ValidateSecResponse>();
    }

    #[test]
    fn should_return_default_context_data_when_called_with_state_reference() {
        let validate_state = &ValidateSecResponse::default();
        let ref_to_validate_state = &ValidateSecResponse::default();

        let expected_result = validate_state.get_context_data();

        let result = ref_to_validate_state.get_context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_validate_state = &mut ValidateSecResponse::default();

        let expected_result = false;

        let result = ref_to_validate_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_reference_state()
     {
        let ref_to_validate_state = &ValidateSecResponse::default();
        let _result = ref_to_validate_state
            .get_output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_default_state_data_as_input_data_when_reference_validate_state_in_initial_state()
     {
        let ref_to_validate_state = &ValidateSecResponse::default();

        let expected_result = &ValidateSecResponseInput::default();

        let result = ref_to_validate_state.get_input_data();

        assert_eq!(result, expected_result);
    }
}
