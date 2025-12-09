//! # `ValidateSecResponseInputData` Module
//!
//! This module defines the input data structure and updater patterns for the `ValidateSecResponse` state
//! within the SEC extraction state machine. It provides types and builders for representing and updating
//! the SEC response objects required for validating HTTP responses from SEC API endpoints.
//!
//! ## Types
//! - [`ValidateSecResponseInputData`]: Holds the raw SEC response to be validated by the validation state.
//! - [`ValidateSecResponseInputDataUpdater`]: Updater type for modifying the input data in a controlled manner.
//! - [`ValidateSecResponseInputDataUpdaterBuilder`]: Builder for constructing updater instances with optional fields.
//!
//! ## Integration
//! - Implements [`StateData`](state_maschine::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`ValidateSecResponse`](crate::implementations::states::extract::validate_sec_response) to receive and update validation parameters.
//!
//! ## Usage
//! This module is intended for use in the validation phase of SEC API responses. It supports builder-based updates and
//! integrates with the state machine's updater and state data traits for robust, testable workflows.
//!
//! ## See Also
//! - [`crate::shared::sec_response`]: Utilities for SEC response handling.
//! - [`state_maschine::prelude::StateData`]: Trait for state data integration.
//!
//! ## Examples
//! See the unit tests in this module for usage patterns and updater logic.

use std::fmt;

use crate::error::State as StateError;
use crate::shared::sec_response::SecResponse;
use crate::traits::state_machine::state::StateData;

use state_maschine::prelude::StateData as SMStateData;

/// Input data for validating SEC API responses.
///
/// This struct holds the raw SEC response object that will be validated for proper
/// status codes, content types, and JSON structure. It is designed to be used as part
/// of the SEC document extraction workflow, and supports builder-based updates and
/// integration with the state machine framework.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Default)]
pub struct ValidateSecResponseInputData {
    /// The SEC response that will be validated.
    pub sec_response: SecResponse,
}

impl ValidateSecResponseInputData {
    /// Creates a new instance of the input data for validating SEC responses.
    ///
    /// # Arguments
    ///
    /// * `sec_response` - The raw [`SecResponse`] to be validated.
    ///
    /// # Returns
    ///
    /// Returns a new [`ValidateSecResponseInputData`] instance ready for state processing.
    #[must_use]
    pub const fn new(sec_response: SecResponse) -> Self {
        Self { sec_response }
    }

    /// Returns a reference to the SEC response.
    ///
    /// # Returns
    ///
    /// A reference to the [`SecResponse`] that will be validated.
    #[must_use]
    pub const fn sec_response(&self) -> &SecResponse {
        &self.sec_response
    }
}

impl StateData for ValidateSecResponseInputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(sec_response) = updates.sec_response {
            self.sec_response = sec_response;
        }
        Ok(())
    }
}

impl SMStateData for ValidateSecResponseInputData {
    type UpdateType = ValidateSecResponseInputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }

    /// This method exists to satisfy the `SMStateData` trait bound from the `state_maschine` crate.
    /// All actual state updates are handled by the `StateData::update_state` implementation above.
    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl fmt::Display for ValidateSecResponseInputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tInput Data: {}", self.sec_response)
    }
}

/// Updater for modifying [`ValidateSecResponseInputData`] in a controlled manner.
///
/// This struct allows for partial updates to input data fields while maintaining
/// type safety and avoiding unnecessary allocations for unchanged fields.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateSecResponseInputDataUpdater {
    /// Optional new SEC response to replace the current one.
    pub sec_response: Option<SecResponse>,
}

/// Builder for constructing [`ValidateSecResponseInputDataUpdater`] instances.
///
/// This builder provides a fluent API for constructing updaters with only
/// the fields that need to be changed, following the builder pattern.
pub struct ValidateSecResponseInputDataUpdaterBuilder {
    sec_response: Option<SecResponse>,
}

impl ValidateSecResponseInputDataUpdaterBuilder {
    /// Creates a new builder with no fields set to be updated.
    ///
    /// # Returns
    ///
    /// A new [`ValidateSecResponseInputDataUpdaterBuilder`] with all fields set to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self { sec_response: None }
    }

    /// Sets the SEC response to be updated.
    ///
    /// # Arguments
    ///
    /// * `sec_response` - The new [`SecResponse`] to set in the input data.
    ///
    /// # Returns
    ///
    /// The builder instance with the SEC response field set for update.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn sec_response(mut self, sec_response: SecResponse) -> Self {
        self.sec_response = Some(sec_response);
        self
    }

    /// Builds the updater with the configured fields.
    ///
    /// # Returns
    ///
    /// A new [`ValidateSecResponseInputDataUpdater`] with the fields set by this builder.
    #[must_use]
    pub fn build(self) -> ValidateSecResponseInputDataUpdater {
        ValidateSecResponseInputDataUpdater {
            sec_response: self.sec_response,
        }
    }
}

impl Default for ValidateSecResponseInputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::sec_response::{ContentType, SecResponse};
    use pretty_assertions::assert_eq;
    use reqwest::StatusCode;
    use std::collections::HashMap;
    use std::{fmt::Debug, hash::Hash};

    #[test]
    fn should_create_new_input_data_with_provided_response() {
        let sec_response = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK1234567890.json",
            )
            .expect("Valid URL"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"test\": \"data\"}"),
        };

        let expected_result = &ValidateSecResponseInputData::new(sec_response.clone());

        let result = &ValidateSecResponseInputData::new(sec_response);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_sec_response_reference_when_accessing_sec_response() {
        let sec_response = SecResponse::default();
        let input_data = ValidateSecResponseInputData::new(sec_response.clone());

        let expected_result = &sec_response;

        let result = input_data.sec_response();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_sec_response_when_updater_contains_sec_response() {
        let original_response = SecResponse::default();
        let new_response = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK9999999999.json",
            )
            .expect("Valid URL"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"updated\": true}"),
        };
        let mut input_data = ValidateSecResponseInputData::new(original_response);
        let updater = ValidateSecResponseInputDataUpdaterBuilder::new()
            .sec_response(new_response.clone())
            .build();

        let expected_result = &ValidateSecResponseInputData::new(new_response);

        StateData::update_state(&mut input_data, updater)
            .expect("Update with valid value should succeed");
        let result = input_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_update_fields_when_updater_is_empty() {
        let sec_response = SecResponse::default();
        let original_input_data = ValidateSecResponseInputData::new(sec_response.clone());
        let mut input_data = original_input_data.clone();
        let updater = ValidateSecResponseInputDataUpdaterBuilder::new().build();

        let expected_result = &ValidateSecResponseInputData::default();

        StateData::update_state(&mut input_data, updater)
            .expect("Update with valid value should succeed");
        let result = input_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_default_input_data_when_default_is_called() {
        let expected_result = ValidateSecResponseInputData {
            sec_response: SecResponse::default(),
        };

        let result = ValidateSecResponseInputData::default();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_input_data_reference_when_accessing_state() {
        let sec_response = SecResponse::default();
        let input_data = ValidateSecResponseInputData::new(sec_response);

        let expected_result = &input_data;
        let result = input_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_sec_response_to_latest_specified_value_when_multiple_updates_in_builder() {
        let original_response = SecResponse::default();
        let intermediate_response = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK5555555555.json",
            )
            .expect("Valid URL"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"intermediate\": true}"),
        };
        let final_response = SecResponse {
            url: reqwest::Url::parse(
                "https://data.sec.gov/api/xbrl/companyfacts/CIK9999999999.json",
            )
            .expect("Valid URL"),
            status: StatusCode::OK,
            headers: HashMap::new(),
            content_type: ContentType::Json,
            body: String::from("{\"final\": true}"),
        };
        let mut input_data = ValidateSecResponseInputData::new(original_response);
        let updater = ValidateSecResponseInputDataUpdaterBuilder::new()
            .sec_response(intermediate_response)
            .sec_response(final_response.clone())
            .build();

        let expected_result = &ValidateSecResponseInputData::new(final_response);

        StateData::update_state(&mut input_data, updater)
            .expect("Update with valid value should succeed");
        let result = input_data.get_state();

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_input_data_trait() {
        implements_auto_traits::<ValidateSecResponseInputData>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_input_data_trait() {
        implements_send::<ValidateSecResponseInputData>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_input_data_trait() {
        implements_sync::<ValidateSecResponseInputData>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_input_data_trait() {
        implements_send::<ValidateSecResponseInputData>();
        implements_sync::<ValidateSecResponseInputData>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_input_data_trait() {
        implements_sized::<ValidateSecResponseInputData>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_input_data_trait() {
        implements_hash::<ValidateSecResponseInputData>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_input_data_trait() {
        implements_partial_eq::<ValidateSecResponseInputData>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_input_data_trait() {
        implements_eq::<ValidateSecResponseInputData>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_input_data_trait() {
        implements_partial_ord::<ValidateSecResponseInputData>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_input_data_trait() {
        implements_ord::<ValidateSecResponseInputData>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_input_data_trait() {
        implements_default::<ValidateSecResponseInputData>();
    }

    #[test]
    const fn should_implement_debug_when_implementing_input_data_trait() {
        const fn implements_debug<T: Debug>() {}
        implements_debug::<ValidateSecResponseInputData>();
    }

    #[test]
    const fn should_implement_clone_when_implementing_input_data_trait() {
        const fn implements_clone<T: Clone>() {}
        implements_clone::<ValidateSecResponseInputData>();
    }

    #[test]
    const fn should_implement_unpin_when_implementing_input_data_trait() {
        const fn implements_unpin<T: Unpin>() {}
        implements_unpin::<ValidateSecResponseInputData>();
    }
}
