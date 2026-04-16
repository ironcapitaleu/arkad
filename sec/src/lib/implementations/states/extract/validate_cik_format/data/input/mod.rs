//! # `ValidateCikFormatInput` Module
//!
//! This module defines the input data structure and updater patterns for the `ValidateCikFormat` state
//! within the SEC extraction state machine. It provides types and builders for representing and updating
//! the raw Central Index Key (CIK) input, which is validated as part of the SEC document extraction workflow.
//!
//! ## Types
//! - [`ValidateCikFormatInput`]: Holds the unvalidated CIK string to be processed by the validation state.
//! - [`ValidateCikFormatInputUpdater`]: Updater type for modifying the input data in a controlled manner.
//! - [`ValidateCikFormatInputUpdaterBuilder`]: Builder for constructing updater instances with optional fields.
//!
//! ## Integration
//! - Implements [`StateData`](state_maschine::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`ValidateCikFormat`](crate::implementations::states::extract::validate_cik_format) to receive and update CIK input.
//!
//! ## Usage
//! This module is intended for use in the input phase of CIK validation. It supports builder-based updates and
//! integrates with the state machine's updater and state data traits for robust, testable workflows.
//!
//! ## See Also
//! - [`output`](super::output): Output data structure for validated CIKs.
//! - [`crate::shared::cik`]: Utilities for CIK parsing and validation.
//! - [`state_maschine::prelude::StateData`]: Trait for state data integration.
//!
//! ## Examples
//! See the unit tests in this module for usage patterns and updater logic.

use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Input data for validating the format of a CIK.
///
/// This struct holds the raw, unvalidated Central Index Key (CIK) string
/// that will be processed by the `ValidateCikFormat` state. It is designed
/// to be used as part of the SEC document extraction workflow, and supports
/// builder-based updates and integration with the state machine framework.
pub struct ValidateCikFormatInput {
    /// The unvalidated CIK string provided for validation.
    pub raw_cik: String,
}

impl ValidateCikFormatInput {
    /// Creates a new instance of the input data for the CIK format validation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::validate_cik_format::data::input::ValidateCikFormatInput;
    ///
    /// let validation_input_data = ValidateCikFormatInput::new("1067983");
    /// ```
    pub fn new(cik: impl Into<String>) -> Self {
        Self {
            raw_cik: cik.into(),
        }
    }

    /// Returns a reference to the raw CIK string.
    #[must_use]
    pub const fn cik(&self) -> &String {
        &self.raw_cik
    }
}

impl StateData for ValidateCikFormatInput {
    /// Updates the state data using the provided updater.
    ///
    /// If `raw_cik` is `Some`, updates the CIK string; otherwise, leaves it unchanged.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(cik) = updates.raw_cik {
            self.raw_cik = cik;
        }
        Ok(())
    }
}

impl SMStateData for ValidateCikFormatInput {
    type UpdateType = ValidateCikFormatInputUpdater;

    /// Returns a reference to the current state data, which represents the input data of this state.
    fn state(&self) -> &Self {
        self
    }

    /// Provided by `SecStateData` trait. Not used in this context.
    fn update_state(&mut self, _updates: Self::UpdateType) {
        // This method is not used in this context.
    }
}

impl Default for ValidateCikFormatInput {
    /// Returns a default input using the CIK for Berkshire Hathaway (CIK: 1067983).
    fn default() -> Self {
        Self {
            raw_cik: BERKSHIRE_HATHAWAY_CIK_RAW.to_string(),
        }
    }
}

impl fmt::Display for ValidateCikFormatInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tCIK: {}", self.raw_cik,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for [`ValidateCikFormatInput`].
///
/// This struct is used to specify updates to the input data in a controlled, partial manner.
/// Fields set to `None` will not be updated. Used in conjunction with the state machine's
/// update mechanism to ensure safe and explicit state transitions.
pub struct ValidateCikFormatInputUpdater {
    /// Optional new value for the raw CIK string.
    pub raw_cik: Option<String>,
}

impl ValidateCikFormatInputUpdater {
    /// Creates a new builder for constructing [`ValidateCikFormatInputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ValidateCikFormatInputUpdaterBuilder {
        ValidateCikFormatInputUpdaterBuilder::new()
    }
}

/// Builder for [`ValidateCikFormatInputUpdater`].
///
/// This builder allows for ergonomic and explicit construction of updater instances,
/// supporting method chaining and optional fields. Use `.build()` to produce the updater.
pub struct ValidateCikFormatInputUpdaterBuilder {
    raw_cik: Option<String>,
}
impl ValidateCikFormatInputUpdaterBuilder {
    /// Creates a new updater builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self { raw_cik: None }
    }

    /// Sets the raw CIK value to the one to be updated to.
    ///
    /// # Arguments
    ///
    /// * `cik` - The new raw CIK value as any type implementing `Into<String>`.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: impl Into<String>) -> Self {
        self.raw_cik = Some(cik.into());
        self
    }

    /// Builds the updater instance from the builder.
    #[must_use]
    pub fn build(self) -> ValidateCikFormatInputUpdater {
        ValidateCikFormatInputUpdater {
            raw_cik: self.raw_cik,
        }
    }
}

impl Default for ValidateCikFormatInputUpdaterBuilder {
    /// Returns a new updater builder with no fields set.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::{assert_eq, assert_ne};

    use super::{
        BERKSHIRE_HATHAWAY_CIK_RAW, ValidateCikFormatInput, ValidateCikFormatInputUpdaterBuilder,
    };
    use crate::traits::state_machine::state::StateData;
    use state_maschine::prelude::StateData as SMStateData;

    #[test]
    fn should_return_reference_to_default_validation_state_data_when_initialized_with_default() {
        let default_validation_state_data = ValidateCikFormatInput::default();

        let expected_result = &ValidateCikFormatInput::default();

        let result = default_validation_state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let validation_state_data = &ValidateCikFormatInput::new("0000000000");

        let default_validation_state_data = &ValidateCikFormatInput::default();

        let result = validation_state_data.state();

        assert_ne!(result, default_validation_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_string_when_update_contains_specified_string() {
        let mut state_data = ValidateCikFormatInput::default();
        let update = ValidateCikFormatInputUpdaterBuilder::default()
            .cik("12345")
            .build();

        let expected_result = &ValidateCikFormatInput::new("12345");

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");

        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut state_data = ValidateCikFormatInput::default();
        let update = ValidateCikFormatInputUpdaterBuilder::default()
            .cik("1234567890")
            .cik("0000000000")
            .build();

        let expected_result = &ValidateCikFormatInput::new("0000000000");

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = ValidateCikFormatInput::default();
        let empty_update = ValidateCikFormatInputUpdaterBuilder::default().build();

        let expected_result = &ValidateCikFormatInput::default();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_cik_when_validation_input_data_initialized_with_default() {
        let validation_state_data = &ValidateCikFormatInput::default();

        let expected_result = &BERKSHIRE_HATHAWAY_CIK_RAW.to_string();

        let result = validation_state_data.state().cik();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_input_data_trait() {
        implements_auto_traits::<ValidateCikFormatInput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_input_data_trait() {
        implements_send::<ValidateCikFormatInput>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_input_data_trait() {
        implements_sync::<ValidateCikFormatInput>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_input_data_trait() {
        implements_send::<ValidateCikFormatInput>();
        implements_sync::<ValidateCikFormatInput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_input_data_trait() {
        implements_sized::<ValidateCikFormatInput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_input_data_trait() {
        implements_hash::<ValidateCikFormatInput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_input_data_trait() {
        implements_partial_eq::<ValidateCikFormatInput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_input_data_trait() {
        implements_eq::<ValidateCikFormatInput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_input_data_trait() {
        implements_partial_ord::<ValidateCikFormatInput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_input_data_trait() {
        implements_ord::<ValidateCikFormatInput>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_input_data_trait() {
        implements_default::<ValidateCikFormatInput>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_input_data_trait() {
        implements_debug::<ValidateCikFormatInput>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_input_data_trait() {
        implements_clone::<ValidateCikFormatInput>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_input_data_trait() {
        implements_unpin::<ValidateCikFormatInput>();
    }
}
