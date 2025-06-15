//! # `ValidateCikFormatInputData` Module
//!
//! This module defines the input data structure and updater patterns for the `ValidateCikFormat` state
//! within the SEC extraction state machine. It provides types and builders for representing and updating
//! the raw Central Index Key (CIK) input, which is validated as part of the SEC document extraction workflow.
//!
//! ## Types
//! - [`ValidateCikFormatInputData`]: Holds the unvalidated CIK string to be processed by the validation state.
//! - [`ValidateCikFormatInputDataUpdater`]: Updater type for modifying the input data in a controlled manner.
//! - [`ValidateCikFormatInputDataUpdaterBuilder`]: Builder for constructing updater instances with optional fields.
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
//! - [`vcf_output_data`](super::vcf_output_data): Output data structure for validated CIKs.
//! - [`crate::shared::cik`]: Utilities for CIK parsing and validation.
//! - [`state_maschine::prelude::StateData`]: Trait for state data integration.
//!
//! ## Examples
//! See the unit tests in this module for usage patterns and updater logic.

use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Input data for validating the format of a CIK.
///
/// This struct holds the raw, unvalidated Central Index Key (CIK) string
/// that will be processed by the `ValidateCikFormat` state. It is designed
/// to be used as part of the SEC document extraction workflow, and supports
/// builder-based updates and integration with the state machine framework.
pub struct ValidateCikFormatInputData {
    /// The unvalidated CIK string provided for validation.
    pub raw_cik: String,
}

impl ValidateCikFormatInputData {
    /// Creates a new instance of the input data for the CIK format validation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::validate_cik_format::vcf_data::vcf_input_data::ValidateCikFormatInputData;
    ///
    /// let validation_input_data = ValidateCikFormatInputData::new("1067983");
    /// ```
    pub fn new(cik: &(impl ToString + ?Sized)) -> Self {
        Self {
            raw_cik: cik.to_string(),
        }
    }

    /// Returns a reference to the raw CIK string.
    #[must_use]
    pub const fn cik(&self) -> &String {
        &self.raw_cik
    }
}

impl StateData for ValidateCikFormatInputData {
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

impl SMStateData for ValidateCikFormatInputData {
    type UpdateType = ValidateCikFormatInputDataUpdater;

    /// Returns a reference to the current state data, which represents the input data of this state.
    fn get_state(&self) -> &Self {
        self
    }

    /// Provided by `SecStateData` trait. Not used in this context.
    fn update_state(&mut self, _updates: Self::UpdateType) {
        // This method is not used in this context.
    }
}

impl Default for ValidateCikFormatInputData {
    /// Returns a default input using the CIK for Berkshire Hathaway (CIK: 1067983).
    fn default() -> Self {
        Self {
            raw_cik: BERKSHIRE_HATHAWAY_CIK.to_string(),
        }
    }
}

impl fmt::Display for ValidateCikFormatInputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tCIK: {}", self.raw_cik,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for [`ValidateCikFormatInputData`].
///
/// This struct is used to specify updates to the input data in a controlled, partial manner.
/// Fields set to `None` will not be updated. Used in conjunction with the state machine's
/// update mechanism to ensure safe and explicit state transitions.
pub struct ValidateCikFormatInputDataUpdater {
    /// Optional new value for the raw CIK string.
    pub raw_cik: Option<String>,
}

/// Builder for [`ValidateCikFormatInputDataUpdater`].
///
/// This builder allows for ergonomic and explicit construction of updater instances,
/// supporting method chaining and optional fields. Use `.build()` to produce the updater.
pub struct ValidateCikFormatInputDataUpdaterBuilder {
    raw_cik: Option<String>,
}
impl ValidateCikFormatInputDataUpdaterBuilder {
    /// Creates a new updater builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self { raw_cik: None }
    }

    /// Sets the raw CIK value to the one to be updated to.
    ///
    /// # Arguments
    ///
    /// * `cik` - The new raw CIK value as any type implementing `ToString`.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: &(impl ToString + ?Sized)) -> Self {
        self.raw_cik = Some(cik.to_string());
        self
    }

    /// Builds the updater instance from the builder.
    #[must_use]
    pub fn build(self) -> ValidateCikFormatInputDataUpdater {
        ValidateCikFormatInputDataUpdater {
            raw_cik: self.raw_cik,
        }
    }
}

impl Default for ValidateCikFormatInputDataUpdaterBuilder {
    /// Returns a new updater builder with no fields set.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BERKSHIRE_HATHAWAY_CIK, ValidateCikFormatInputData,
        ValidateCikFormatInputDataUpdaterBuilder,
    };
    use crate::traits::state_machine::state::StateData;
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::StateData as SMStateData;
    use std::{fmt::Debug, hash::Hash};

    #[test]
    fn should_return_reference_to_default_validation_state_data_when_initialized_with_default() {
        let default_validation_state_data = ValidateCikFormatInputData::default();

        let expected_result = &ValidateCikFormatInputData::default();

        let result = default_validation_state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let validation_state_data = &ValidateCikFormatInputData::new("0000000000");

        let default_validation_state_data = &ValidateCikFormatInputData::default();

        let result = validation_state_data.get_state();

        assert_ne!(result, default_validation_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_string_when_update_contains_specified_string() {
        let mut state_data = ValidateCikFormatInputData::default();
        let update = ValidateCikFormatInputDataUpdaterBuilder::default()
            .cik("12345")
            .build();

        let expected_result = &ValidateCikFormatInputData::new("12345");

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed.");

        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut state_data = ValidateCikFormatInputData::default();
        let update = ValidateCikFormatInputDataUpdaterBuilder::default()
            .cik("1234567890")
            .cik("0000000000")
            .build();

        let expected_result = &ValidateCikFormatInputData::new("0000000000");

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = ValidateCikFormatInputData::default();
        let empty_update = ValidateCikFormatInputDataUpdaterBuilder::default().build();

        let expected_result = &ValidateCikFormatInputData::default();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_cik_when_validation_input_data_initialized_with_default() {
        let validation_state_data = &ValidateCikFormatInputData::default();

        let expected_result = &BERKSHIRE_HATHAWAY_CIK.to_string();

        let result = validation_state_data.get_state().cik();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_trait() {
        implements_auto_traits::<ValidateCikFormatInputData>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_trait() {
        implements_send::<ValidateCikFormatInputData>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_trait() {
        implements_sync::<ValidateCikFormatInputData>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_trait() {
        implements_send::<ValidateCikFormatInputData>();
        implements_sync::<ValidateCikFormatInputData>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_trait() {
        implements_sized::<ValidateCikFormatInputData>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_trait() {
        implements_hash::<ValidateCikFormatInputData>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_trait() {
        implements_partial_eq::<ValidateCikFormatInputData>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_trait() {
        implements_eq::<ValidateCikFormatInputData>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_trait() {
        implements_partial_ord::<ValidateCikFormatInputData>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_trait() {
        implements_ord::<ValidateCikFormatInputData>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_state_trait() {
        implements_default::<ValidateCikFormatInputData>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_trait() {
        implements_debug::<ValidateCikFormatInputData>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_trait() {
        implements_clone::<ValidateCikFormatInputData>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_trait() {
        implements_unpin::<ValidateCikFormatInputData>();
    }
}
