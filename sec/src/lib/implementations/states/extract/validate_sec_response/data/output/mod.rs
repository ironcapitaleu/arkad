//! # `ValidateSecResponseOutput` Module
//!
//! This module defines the output data structure and updater patterns for the `ValidateSecResponse` state
//! within the SEC extraction state machine. It provides types and builders for representing and updating
//! the validated JSON response produced by the validation process.
//!
//! ## Types
//! - [`ValidateSecResponseOutput`]: Holds the validated JSON response produced by the validation state.
//! - [`ValidateSecResponseOutputUpdater`]: Updater type for modifying the output data in a controlled manner.
//! - [`ValidateSecResponseOutputUpdaterBuilder`]: Builder for constructing updater instances with optional fields.
//!
//! ## Integration
//! - Implements [`StateData`](state_maschine::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`ValidateSecResponse`](crate::implementations::states::extract::validate_sec_response) to store and update validation results.
//!
//! ## Usage
//! This module is intended for use after successful validation of SEC API responses. It supports builder-based updates and
//! integrates with the state machine's updater and state data traits for robust, testable workflows.
//!
//! ## See Also
//! - [`crate::shared::json_response`]: Utilities for JSON response handling.
//! - [`state_maschine::prelude::StateData`]: Trait for state data integration.
//!
//! ## Examples
//! See the unit tests in this module for usage patterns and updater logic.

use std::fmt;

use crate::error::State as StateError;
use crate::shared::json_response::JsonResponse;
use crate::traits::state_machine::state::StateData;

use state_maschine::prelude::StateData as SMStateData;

/// Output data for validated SEC API responses.
///
/// This struct holds the validated JSON response produced after validating an SEC
/// HTTP response for proper status codes, content types, and JSON structure. It is
/// designed to be used as part of the SEC document extraction workflow, and supports
/// builder-based updates and integration with the state machine framework.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Default)]
pub struct ValidateSecResponseOutput {
    /// The validated JSON response containing parsed SEC data.
    pub validated_sec_response: JsonResponse,
}

impl ValidateSecResponseOutput {
    /// Creates a new instance of the output data for validated SEC responses.
    ///
    /// # Arguments
    ///
    /// * `validated_sec_response` - The validated [`JsonResponse`] containing parsed JSON data.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing the new [`ValidateSecResponseOutput`] instance,
    /// or a [`StateError`] if the data is invalid.
    ///
    /// # Errors
    ///
    /// Returns a `StateError` if the provided data is invalid (currently always succeeds).
    pub const fn new(validated_sec_response: JsonResponse) -> Result<Self, StateError> {
        Ok(Self {
            validated_sec_response,
        })
    }

    /// Returns a reference to the validated JSON response.
    ///
    /// # Returns
    ///
    /// A reference to the [`JsonResponse`] containing the validated SEC data.
    #[must_use]
    pub const fn validated_sec_response(&self) -> &JsonResponse {
        &self.validated_sec_response
    }
}

impl StateData for ValidateSecResponseOutput {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(validated_sec_response) = updates.validated_sec_response {
            self.validated_sec_response = validated_sec_response;
        }
        Ok(())
    }
}

impl SMStateData for ValidateSecResponseOutput {
    type UpdateType = ValidateSecResponseOutputUpdater;

    fn state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl fmt::Display for ValidateSecResponseOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tOutput Data: {}", self.validated_sec_response)
    }
}

/// Updater for modifying [`ValidateSecResponseOutput`] in a controlled manner.
///
/// This struct allows for partial updates to output data fields while maintaining
/// type safety and avoiding unnecessary allocations for unchanged fields.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateSecResponseOutputUpdater {
    /// Optional new validated response to replace the current one.
    pub validated_sec_response: Option<JsonResponse>,
}

impl ValidateSecResponseOutputUpdater {
    /// Creates a new builder for constructing [`ValidateSecResponseOutputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ValidateSecResponseOutputUpdaterBuilder {
        ValidateSecResponseOutputUpdaterBuilder::new()
    }
}

/// Builder for constructing [`ValidateSecResponseOutputUpdater`] instances.
///
/// This builder provides a fluent API for constructing updaters with only
/// the fields that need to be changed, following the builder pattern.
pub struct ValidateSecResponseOutputUpdaterBuilder {
    validated_sec_response: Option<JsonResponse>,
}

impl ValidateSecResponseOutputUpdaterBuilder {
    /// Creates a new builder with no fields set to be updated.
    ///
    /// # Returns
    ///
    /// A new [`ValidateSecResponseOutputUpdaterBuilder`] with all fields set to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            validated_sec_response: None,
        }
    }

    /// Sets the validated response to be updated.
    ///
    /// # Arguments
    ///
    /// * `validated_sec_response` - The new [`JsonResponse`] to set in the output data.
    ///
    /// # Returns
    ///
    /// The builder instance with the validated response field set for update.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn validated_sec_response(mut self, validated_sec_response: JsonResponse) -> Self {
        self.validated_sec_response = Some(validated_sec_response);
        self
    }

    /// Builds the updater with the configured fields.
    ///
    /// # Returns
    ///
    /// A new [`ValidateSecResponseOutputUpdater`] with the fields set by this builder.
    #[must_use]
    pub fn build(self) -> ValidateSecResponseOutputUpdater {
        ValidateSecResponseOutputUpdater {
            validated_sec_response: self.validated_sec_response,
        }
    }
}

impl Default for ValidateSecResponseOutputUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use reqwest::StatusCode;

    use super::*;
    use crate::shared::json_response::JsonResponse;
    use crate::shared::sec_response::{ContentType, SecResponse};

    #[test]
    fn should_create_new_output_data_with_provided_response() {
        let sec_response = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK1234567890.json",
            )
            .expect("Hardcoded URL should always be valid"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"test\": \"data\"}"),
        };
        let json_response = JsonResponse::from_sec_response(&sec_response)
            .expect("Valid SEC response string should parse to JSON successfully");

        let expected_result = ValidateSecResponseOutput::new(json_response.clone())
            .expect("Valid JSON response should create output successfully");

        let result = ValidateSecResponseOutput::new(json_response)
            .expect("Valid JSON response should create output successfully");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_json_response_reference_when_accessing_validated_sec_response() {
        let sec_response = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK1234567890.json",
            )
            .expect("Hardcoded URL should always be valid"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"data\": [1,2,3]}"),
        };
        let json_response = JsonResponse::from_sec_response(&sec_response)
            .expect("Should create valid JSON response if SEC response is valid");
        let output_data = ValidateSecResponseOutput::new(json_response.clone())
            .expect("Should create output data when valid JSON response is provided");

        let expected_result = &json_response;

        let result = output_data.validated_sec_response();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_validated_sec_response_when_updater_contains_validated_sec_response() {
        let original_response = JsonResponse::default();
        let sec_response = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK9999999999.json",
            )
            .expect("URL should be valid"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"updated\": true}"),
        };
        let new_response = JsonResponse::from_sec_response(&sec_response)
            .expect("Valid SEC response string should parse to JSON successfully");
        let mut output_data = ValidateSecResponseOutput::new(original_response)
            .expect("Valid JSON response should create output successfully");
        let updater = ValidateSecResponseOutputUpdater::builder()
            .validated_sec_response(new_response.clone())
            .build();

        let expected_result = &ValidateSecResponseOutput::new(new_response)
            .expect("Valid JSON response should create output successfully");

        StateData::update_state(&mut output_data, updater)
            .expect("Valid update should always succeed with valid input");
        let result = output_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_update_fields_when_updater_is_empty() {
        let json_response = JsonResponse::default();
        let original_output_data = ValidateSecResponseOutput::new(json_response.clone())
            .expect("Valid JSON response should create output successfully");
        let mut output_data = original_output_data.clone();
        let updater = ValidateSecResponseOutputUpdater::builder().build();

        let expected_result = &ValidateSecResponseOutput::default();

        StateData::update_state(&mut output_data, updater)
            .expect("Valid update should always succeed with valid input");
        let result = output_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_default_output_data_when_default_is_called() {
        let expected_result = ValidateSecResponseOutput {
            validated_sec_response: JsonResponse::default(),
        };

        let result = ValidateSecResponseOutput::default();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_output_data_reference_when_accessing_state() {
        let json_response = JsonResponse::default();
        let output_data = ValidateSecResponseOutput::new(json_response)
            .expect("Valid JSON response should create output successfully");

        let expected_result = &output_data;
        let result = output_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_validated_sec_response_to_latest_specified_value_when_multiple_updates_in_builder()
     {
        let original_response = JsonResponse::default();
        let sec_response1 = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK5555555555.json",
            )
            .expect("Hardcoded URL should always be valid"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"intermediate\": true}"),
        };
        let intermediate_response = JsonResponse::from_sec_response(&sec_response1)
            .expect("Valid SEC response string should parse to JSON successfully");
        let sec_response2 = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK9999999999.json",
            )
            .expect("Hardcoded URL should always be valid"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"final\": true}"),
        };
        let final_response = JsonResponse::from_sec_response(&sec_response2)
            .expect("Valid SEC response string should parse to JSON successfully");
        let mut output_data = ValidateSecResponseOutput::new(original_response)
            .expect("Valid JSON response should create output successfully");
        let updater = ValidateSecResponseOutputUpdater::builder()
            .validated_sec_response(intermediate_response)
            .validated_sec_response(final_response.clone())
            .build();

        let expected_result = &ValidateSecResponseOutput::new(final_response)
            .expect("Valid JSON response should create output successfully");

        StateData::update_state(&mut output_data, updater)
            .expect("Valid update should always succeed with valid input");
        let result = output_data.state();

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_output_data_trait() {
        implements_auto_traits::<ValidateSecResponseOutput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_output_data_trait() {
        implements_send::<ValidateSecResponseOutput>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_output_data_trait() {
        implements_sync::<ValidateSecResponseOutput>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_output_data_trait() {
        implements_send::<ValidateSecResponseOutput>();
        implements_sync::<ValidateSecResponseOutput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_output_data_trait() {
        implements_sized::<ValidateSecResponseOutput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_output_data_trait() {
        implements_hash::<ValidateSecResponseOutput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_output_data_trait() {
        implements_partial_eq::<ValidateSecResponseOutput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_output_data_trait() {
        implements_eq::<ValidateSecResponseOutput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_output_data_trait() {
        implements_partial_ord::<ValidateSecResponseOutput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_output_data_trait() {
        implements_ord::<ValidateSecResponseOutput>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_output_data_trait() {
        implements_default::<ValidateSecResponseOutput>();
    }

    #[test]
    const fn should_implement_debug_when_implementing_output_data_trait() {
        const fn implements_debug<T: Debug>() {}
        implements_debug::<ValidateSecResponseOutput>();
    }

    #[test]
    const fn should_implement_clone_when_implementing_output_data_trait() {
        const fn implements_clone<T: Clone>() {}
        implements_clone::<ValidateSecResponseOutput>();
    }

    #[test]
    const fn should_implement_unpin_when_implementing_output_data_trait() {
        const fn implements_unpin<T: Unpin>() {}
        implements_unpin::<ValidateSecResponseOutput>();
    }
}
