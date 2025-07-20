//! # `PrepareSecRequestInputData` Module
//!
//! This module defines the input data structure and updater patterns for the `PrepareSecRequest` state
//! within the SEC extraction state machine. It provides types and builders for representing and updating
//! the validated Central Index Key (CIK) and user agent information required for preparing SEC API requests.
//!
//! ## Types
//! - [`PrepareSecRequestInputData`]: Holds the validated CIK and user agent string to be processed by the prepare request state.
//! - [`PrepareSecRequestInputDataUpdater`]: Updater type for modifying the input data in a controlled manner.
//! - [`PrepareSecRequestInputDataUpdaterBuilder`]: Builder for constructing updater instances with optional fields.
//!
//! ## Integration
//! - Implements [`StateData`](state_maschine::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request) to receive and update request parameters.
//!
//! ## Usage
//! This module is intended for use in the preparation phase of SEC API requests. It supports builder-based updates and
//! integrates with the state machine's updater and state data traits for robust, testable workflows.
//!
//! ## See Also
//! - [`crate::shared::cik`]: Utilities for CIK parsing and validation.
//! - [`crate::shared::user_agent`]: Utilities for user agent validation.
//! - [`state_maschine::prelude::StateData`]: Trait for state data integration.
//!
//! ## Examples
//! See the unit tests in this module for usage patterns and updater logic.

use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::cik::Cik;
use crate::traits::state_machine::state::StateData;

/// Input data for preparing SEC API requests.
///
/// This struct holds the validated Central Index Key (CIK) and user agent string
/// that will be used to prepare HTTP requests to SEC API endpoints. It is designed
/// to be used as part of the SEC document extraction workflow, and supports
/// builder-based updates and integration with the state machine framework.
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestInputData {
    /// The validated CIK that will be used for the SEC API request.
    pub validated_cik: Cik,
    /// The user agent string that will be included in the HTTP request headers.
    pub user_agent: String,
}

impl PrepareSecRequestInputData {
    /// Creates a new instance of the input data for preparing SEC requests.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::prepare_sec_request::psr_data::psr_input_data::PrepareSecRequestInputData;
    /// use sec::shared::cik::Cik;
    ///
    /// let cik = Cik::new("1067983").expect("Valid CIK");
    /// let user_agent = "Test Company contact@test.com".to_string();
    /// let input_data = PrepareSecRequestInputData::new(cik, user_agent);
    /// ```
    #[must_use]
    pub const fn new(validated_cik: Cik, user_agent: String) -> Self {
        Self {
            validated_cik,
            user_agent,
        }
    }

    /// Returns a reference to the validated CIK.
    #[must_use]
    pub const fn validated_cik(&self) -> &Cik {
        &self.validated_cik
    }

    /// Returns a reference to the user agent string.
    #[must_use]
    pub const fn user_agent(&self) -> &String {
        &self.user_agent
    }
}

impl StateData for PrepareSecRequestInputData {
    /// Updates the state data using the provided updater.
    ///
    /// If `validated_cik` is `Some`, updates the CIK; if `user_agent` is `Some`, updates the user agent;
    /// otherwise, leaves the respective fields unchanged.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(validated_cik) = updates.validated_cik {
            self.validated_cik = validated_cik;
        }
        if let Some(user_agent) = updates.user_agent {
            self.user_agent = user_agent;
        }
        Ok(())
    }
}

impl SMStateData for PrepareSecRequestInputData {
    type UpdateType = PrepareSecRequestInputDataUpdater;

    /// Returns a reference to the current state data, which represents the input data of this state.
    fn get_state(&self) -> &Self {
        self
    }

    /// Provided by `SecStateData` trait. Not used in this context.
    fn update_state(&mut self, _updates: Self::UpdateType) {
        // This method is not used in this context.
    }
}

impl fmt::Display for PrepareSecRequestInputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tValidated CIK: {}\nUser Agent: {}",
            self.validated_cik, self.user_agent
        )
    }
}

/// Updater for [`PrepareSecRequestInputData`].
///
/// This struct is used to specify updates to the input data in a controlled, partial manner.
/// Fields set to `None` will not be updated. Used in conjunction with the state machine's
/// update mechanism to ensure safe and explicit state transitions.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestInputDataUpdater {
    /// Optional new value for the validated CIK.
    pub validated_cik: Option<Cik>,
    /// Optional new value for the user agent string.
    pub user_agent: Option<String>,
}

/// Builder for [`PrepareSecRequestInputDataUpdater`].
///
/// This builder allows for ergonomic and explicit construction of updater instances,
/// supporting method chaining and optional fields. Use `.build()` to produce the updater.
pub struct PrepareSecRequestInputDataUpdaterBuilder {
    pub validated_cik: Option<Cik>,
    pub user_agent: Option<String>,
}

impl PrepareSecRequestInputDataUpdaterBuilder {
    /// Creates a new updater builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            validated_cik: None,
            user_agent: None,
        }
    }

    /// Sets both the validated CIK and user agent values to be updated.
    ///
    /// # Arguments
    ///
    /// * `validated_cik` - The new validated CIK value.
    /// * `user_agent` - The new user agent string value.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn validated_cik(mut self, validated_cik: Cik, user_agent: String) -> Self {
        self.validated_cik = Some(validated_cik);
        self.user_agent = Some(user_agent);
        self
    }

    /// Sets only the user agent value to be updated.
    ///
    /// # Arguments
    ///
    /// * `user_agent` - The new user agent string value.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Sets only the validated CIK value to be updated.
    ///
    /// # Arguments
    ///
    /// * `validated_cik` - The new validated CIK value.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, validated_cik: Cik) -> Self {
        self.validated_cik = Some(validated_cik);
        self
    }

    /// Builds the updater instance from the builder.
    #[must_use]
    pub fn build(self) -> PrepareSecRequestInputDataUpdater {
        PrepareSecRequestInputDataUpdater {
            validated_cik: self.validated_cik,
            user_agent: self.user_agent,
        }
    }
}

impl Default for PrepareSecRequestInputDataUpdaterBuilder {
    /// Returns a new updater builder with no fields set.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::{assert_eq, assert_ne};

    use super::{PrepareSecRequestInputData, PrepareSecRequestInputDataUpdaterBuilder};
    use crate::shared::cik::Cik;
    use crate::traits::state_machine::state::StateData;
    use state_maschine::prelude::StateData as SMStateData;

    #[test]
    fn should_return_reference_to_default_prepare_state_data_when_initialized_with_default() {
        let default_prepare_state_data = PrepareSecRequestInputData::default();

        let expected_result = &PrepareSecRequestInputData::default();

        let result = default_prepare_state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let user_agent = "Custom Company contact@custom.com".to_string();
        let prepare_state_data = PrepareSecRequestInputData::new(cik, user_agent);

        let default_prepare_state_data = &PrepareSecRequestInputData::default();

        let result = prepare_state_data.get_state();

        assert_ne!(result, default_prepare_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_values_when_update_contains_specified_values() {
        let mut state_data = PrepareSecRequestInputData::default();
        let new_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let new_user_agent = "Updated Company contact@updated.com".to_string();
        let update = PrepareSecRequestInputDataUpdaterBuilder::default()
            .validated_cik(new_cik.clone(), new_user_agent.clone())
            .build();

        let expected_result = &PrepareSecRequestInputData::new(new_cik, new_user_agent);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_only_user_agent_when_update_contains_only_user_agent() {
        let mut state_data = PrepareSecRequestInputData::default();
        let original_cik = state_data.validated_cik.clone();
        let new_user_agent = "New User Agent contact@new.com".to_string();
        let update = PrepareSecRequestInputDataUpdaterBuilder::default()
            .user_agent(new_user_agent.clone())
            .build();

        let expected_result = &PrepareSecRequestInputData::new(original_cik, new_user_agent);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_only_cik_when_update_contains_only_cik() {
        let mut state_data = PrepareSecRequestInputData::default();
        let original_user_agent = state_data.user_agent.clone();
        let new_cik = Cik::new("9876543210").expect("Hardcoded CIK should always be valid.");
        let update = PrepareSecRequestInputDataUpdaterBuilder::default()
            .cik(new_cik.clone())
            .build();

        let expected_result = &PrepareSecRequestInputData::new(new_cik, original_user_agent);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_values_when_multiple_updates_in_builder() {
        let mut state_data = PrepareSecRequestInputData::default();
        let first_cik = Cik::new("1111111111").expect("Hardcoded CIK should always be valid.");
        let first_user_agent = "First Company contact@first.com".to_string();
        let final_cik = Cik::new("2222222222").expect("Hardcoded CIK should always be valid.");
        let final_user_agent = "Final Company contact@final.com".to_string();

        let update = PrepareSecRequestInputDataUpdaterBuilder::default()
            .validated_cik(first_cik, first_user_agent)
            .validated_cik(final_cik.clone(), final_user_agent.clone())
            .build();

        let expected_result = &PrepareSecRequestInputData::new(final_cik, final_user_agent);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = PrepareSecRequestInputData::default();
        let empty_update = PrepareSecRequestInputDataUpdaterBuilder::default().build();

        let expected_result = &PrepareSecRequestInputData::default();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_validated_cik_when_accessor_method_is_called() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let user_agent = "Test Company contact@test.com".to_string();
        let prepare_state_data = PrepareSecRequestInputData::new(cik.clone(), user_agent);

        let expected_result = &cik;

        let result = prepare_state_data.validated_cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_user_agent_when_accessor_method_is_called() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let user_agent = "Test Company contact@test.com".to_string();
        let prepare_state_data = PrepareSecRequestInputData::new(cik, user_agent.clone());

        let expected_result = &user_agent;

        let result = prepare_state_data.user_agent();

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_input_data_trait() {
        implements_auto_traits::<PrepareSecRequestInputData>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_input_data_trait() {
        implements_send::<PrepareSecRequestInputData>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_input_data_trait() {
        implements_sync::<PrepareSecRequestInputData>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_input_data_trait() {
        implements_send::<PrepareSecRequestInputData>();
        implements_sync::<PrepareSecRequestInputData>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_input_data_trait() {
        implements_sized::<PrepareSecRequestInputData>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_input_data_trait() {
        implements_hash::<PrepareSecRequestInputData>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_input_data_trait() {
        implements_partial_eq::<PrepareSecRequestInputData>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_input_data_trait() {
        implements_eq::<PrepareSecRequestInputData>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_input_data_trait() {
        implements_partial_ord::<PrepareSecRequestInputData>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_input_data_trait() {
        implements_ord::<PrepareSecRequestInputData>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_input_data_trait() {
        implements_default::<PrepareSecRequestInputData>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_input_data_trait() {
        implements_debug::<PrepareSecRequestInputData>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_input_data_trait() {
        implements_clone::<PrepareSecRequestInputData>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_input_data_trait() {
        implements_unpin::<PrepareSecRequestInputData>();
    }
}
