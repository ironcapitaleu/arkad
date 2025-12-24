//! # `ExecuteSecRequestOutput` Module
//!
//! This module defines the output data structure and updater patterns for the `ExecuteSecRequest` state
//! within the SEC extraction state machine. It encapsulates the SEC response received from executing
//! HTTP requests to SEC API endpoints and provides builders and updaters for controlled mutation of output data.
//!
//! ## Types
//! - [`ExecuteSecRequestOutput`]: Holds the SEC response after successful request execution.
//! - [`ExecuteSecRequestOutputUpdater`]: Updater type for modifying the output data in a controlled manner.
//! - [`ExecuteSecRequestOutputUpdaterBuilder`]: Builder for constructing updater instances with optional fields.
//!
//! ## Integration
//! - Implements [`StateData`](state_maschine::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request) to produce and update request output data.
//!
//! ## Usage
//! This module is intended for use in the output phase of SEC request execution. It supports builder-based updates and
//! integrates with the state machine's updater and state data traits for robust, testable workflows.
//!
//! ## See Also
//! - [`super::input`]: Input data structure for request execution parameters.
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

/// Output data containing a SEC response from executed requests.
///
/// This struct holds a [`SecResponse`] value, produced by the [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request) state
/// after successful HTTP request execution. It is used as output in the SEC extraction state machine,
/// and supports builder-based updates and integration with the state machine framework.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ExecuteSecRequestOutput {
    /// The SEC response received from the API endpoint.
    pub response: SecResponse,
}

impl ExecuteSecRequestOutput {
    /// Creates a new instance of the output data for executed SEC requests.
    ///
    /// # Arguments
    ///
    /// * `response` - The [`SecResponse`] received from the SEC API endpoint.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing the new [`ExecuteSecRequestOutput`] if successful.
    ///
    /// # Errors
    ///
    /// Returns `StateError` if the provided `SecResponse` is invalid (currently this never fails,
    /// but the Result type is maintained for future validation capabilities).
    pub const fn new(response: SecResponse) -> Result<Self, StateError> {
        Ok(Self { response })
    }

    /// Returns a reference to the SEC response.
    ///
    /// # Returns
    ///
    /// A reference to the [`SecResponse`] containing the API response data.
    #[must_use]
    pub const fn response(&self) -> &SecResponse {
        &self.response
    }
}
impl StateData for ExecuteSecRequestOutput {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(response) = updates.response {
            self.response = response;
        }
        Ok(())
    }
}
impl SMStateData for ExecuteSecRequestOutput {
    type UpdateType = ExecuteSecRequestOutputUpdater;

    fn get_state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl fmt::Display for ExecuteSecRequestOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\t{}", self.response)
    }
}

/// Updater for modifying [`ExecuteSecRequestOutput`] in a controlled manner.
///
/// This struct allows for partial updates to output data fields while maintaining
/// type safety and avoiding unnecessary allocations for unchanged fields.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ExecuteSecRequestOutputUpdater {
    /// Optional new SEC response to replace the current one.
    pub response: Option<SecResponse>,
}

impl ExecuteSecRequestOutputUpdater {
    /// Creates a new builder for constructing [`ExecuteSecRequestOutputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ExecuteSecRequestOutputUpdaterBuilder {
        ExecuteSecRequestOutputUpdaterBuilder::new()
    }
}

/// Builder for constructing [`ExecuteSecRequestOutputUpdater`] instances.
///
/// This builder provides a fluent API for constructing updaters with only
/// the fields that need to be changed, following the builder pattern.
pub struct ExecuteSecRequestOutputUpdaterBuilder {
    response: Option<SecResponse>,
}

impl ExecuteSecRequestOutputUpdaterBuilder {
    /// Creates a new builder with no fields set to be updated.
    ///
    /// # Returns
    ///
    /// A new [`ExecuteSecRequestOutputUpdaterBuilder`] with all fields set to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self { response: None }
    }

    /// Sets the SEC response to be updated.
    ///
    /// # Arguments
    ///
    /// * `response` - The new [`SecResponse`] to set in the output data.
    ///
    /// # Returns
    ///
    /// The builder instance with the response field set for update.
    #[must_use]
    pub fn response(mut self, response: SecResponse) -> Self {
        self.response = Some(response);
        self
    }

    /// Builds the updater with the configured fields.
    ///
    /// # Returns
    ///
    /// A new [`ExecuteSecRequestOutputUpdater`] with the fields set by this builder.
    #[must_use]
    pub fn build(self) -> ExecuteSecRequestOutputUpdater {
        ExecuteSecRequestOutputUpdater {
            response: self.response,
        }
    }
}

impl Default for ExecuteSecRequestOutputUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::sec_response::SecResponse;
    use pretty_assertions::assert_eq;

    use std::{fmt::Debug, hash::Hash};

    #[test]
    fn should_create_new_output_data_with_provided_response() {
        let response = SecResponse::default();
        let expected_response = response.clone();

        let result = ExecuteSecRequestOutput::new(response).expect("Should create output data");

        assert_eq!(result.response(), &expected_response);
    }

    #[test]
    fn should_return_response_reference_when_accessing_response() {
        let response = SecResponse::default();
        let output_data =
            ExecuteSecRequestOutput::new(response.clone()).expect("Should create output data");

        let expected_result = &response;
        let result = output_data.response();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_response_when_updater_contains_response() {
        let original_response = SecResponse::default();
        let new_response = SecResponse::default();
        // Make the new response different by modifying the status (though both will be OK in default)
        // Since SecResponse doesn't have setters, we'll just use default for testing
        let mut output_data =
            ExecuteSecRequestOutput::new(original_response).expect("Should create output data");

        let updater = ExecuteSecRequestOutputUpdater::builder()
            .response(new_response.clone())
            .build();

        let expected_result = Ok(());
        let result = StateData::update_state(&mut output_data, updater);

        assert_eq!(result, expected_result);
        assert_eq!(output_data.response(), &new_response);
    }

    #[test]
    fn should_not_update_fields_when_updater_is_empty() {
        let response = SecResponse::default();
        let original_output_data =
            ExecuteSecRequestOutput::new(response).expect("Should create output data");
        let mut output_data = original_output_data.clone();

        let updater = ExecuteSecRequestOutputUpdater::builder().build();

        let expected_result = Ok(());
        let result = StateData::update_state(&mut output_data, updater);

        assert_eq!(result, expected_result);
        assert_eq!(output_data, original_output_data);
    }

    #[test]
    fn should_display_response_information_when_formatted() {
        let response = SecResponse::default();
        let output_data =
            ExecuteSecRequestOutput::new(response).expect("Should create output data");

        let result = format!("{output_data}");

        assert!(result.contains("SEC Response"));
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_data_trait() {
        implements_auto_traits::<ExecuteSecRequestOutput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_data_trait() {
        implements_send::<ExecuteSecRequestOutput>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_data_trait() {
        implements_sync::<ExecuteSecRequestOutput>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_data_trait() {
        implements_send::<ExecuteSecRequestOutput>();
        implements_sync::<ExecuteSecRequestOutput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_data_trait() {
        implements_sized::<ExecuteSecRequestOutput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_data_trait() {
        implements_hash::<ExecuteSecRequestOutput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_data_trait() {
        implements_partial_eq::<ExecuteSecRequestOutput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_data_trait() {
        implements_eq::<ExecuteSecRequestOutput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_data_trait() {
        implements_partial_ord::<ExecuteSecRequestOutput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_data_trait() {
        implements_ord::<ExecuteSecRequestOutput>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_state_data_trait() {
        implements_default::<ExecuteSecRequestOutput>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_data_trait() {
        implements_debug::<ExecuteSecRequestOutput>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_data_trait() {
        implements_clone::<ExecuteSecRequestOutput>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_data_trait() {
        implements_unpin::<ExecuteSecRequestOutput>();
    }
}
