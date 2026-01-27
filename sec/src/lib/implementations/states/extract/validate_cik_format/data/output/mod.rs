//! # `ValidateCikFormatOutput` Module
//!
//! This module defines the output data structure and updater patterns for the `ValidateCikFormat` state
//! within the SEC extraction state machine. It encapsulates the validated Central Index Key (CIK) and
//! provides builders and updaters for controlled mutation of output data.
//!
//! ## Types
//! - [`ValidateCikFormatOutput`]: Holds the validated CIK after successful format validation.
//! - [`ValidateCikFormatOutputUpdater`]: Updater type for modifying the output data in a controlled manner.
//! - [`ValidateCikFormatOutputUpdaterBuilder`]: Builder for constructing updater instances with optional fields.
//!
//! ## Integration
//! - Implements [`StateData`](state_maschine::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`ValidateCikFormat`](crate::implementations::states::extract::validate_cik_format) to produce and update CIK output data.
//!
//! ## Usage
//! This module is intended for use in the output phase of CIK validation. It supports builder-based updates and
//! integrates with the state machine's updater and state data traits for robust, testable workflows.
//!
//! ## See Also
//! - [`input`](super::input): Input data structure for unvalidated CIKs.
//! - [`crate::shared::cik`]: Utilities for CIK parsing and validation.
//! - [`state_maschine::prelude::StateData`]: Trait for state data integration.
//!
//! ## Examples
//! See the unit tests in this module for usage patterns and updater logic.

use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::error::state_machine::state::InvalidCikFormat;
use crate::shared::cik::{Cik, constants::BERKSHIRE_HATHAWAY_CIK_RAW};
use crate::traits::error::FromDomainError;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Output data containing a validated CIK.
///
/// This struct holds a validated [`Cik`] value, produced by the `ValidateCikFormat` state
/// after successful validation. It is used as output in the SEC extraction state machine,
/// and supports builder-based updates and integration with the state machine framework.
pub struct ValidateCikFormatOutput {
    /// The validated CIK.
    pub validated_cik: Cik,
}

impl ValidateCikFormatOutput {
    /// Creates a new instance of the output data for the CIK validation state.
    /// The output must follow the correct formatting.
    ///
    /// # Errors
    ///
    /// Returns a [`StateError::InvalidCikFormat`] if the CIK is not formatted correctly.
    pub fn new(cik: impl Into<String>) -> Result<Self, StateError> {
        let validated_cik = Self::validate_cik_format(cik.into().as_str())?;
        Ok(Self { validated_cik })
    }

    /// Validates the CIK format and returns a validated [`Cik`] instance.
    ///
    /// # Errors
    ///
    /// Returns a [`StateError::InvalidCikFormat`] if the CIK is not formatted correctly.
    fn validate_cik_format(cik: &str) -> Result<Cik, StateError> {
        Cik::new(&cik).map_err(|e| {
            InvalidCikFormat::from_domain_error(Self::default().state().to_string(), e).into()
        })
    }

    /// Returns a reference to the validated CIK string.
    #[must_use]
    pub const fn cik(&self) -> &String {
        self.validated_cik.value()
    }
}
impl StateData for ValidateCikFormatOutput {
    /// Updates the state data using the provided updater.
    ///
    /// If `cik` is `Some`, updates the validated CIK; otherwise, leaves it unchanged.
    /// Returns an error if the new CIK is invalid.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(cik) = updates.cik {
            match Cik::new(&cik) {
                Ok(valid_cik) => {
                    self.validated_cik = valid_cik;
                    Ok(())
                }
                Err(e) => {
                    Err(InvalidCikFormat::from_domain_error(self.state().to_string(), e).into())
                }
            }
        } else {
            Ok(())
        }
    }
}
impl SMStateData for ValidateCikFormatOutput {
    type UpdateType = ValidateCikFormatOutputUpdater;

    /// Returns a reference to the current state data, , which represents the output data of this state.
    fn state(&self) -> &Self {
        self
    }
    /// Provided by `SecStateData` trait. Not used in this context.
    fn update_state(&mut self, _updates: Self::UpdateType) {
        // This method is not used in this context.
    }
}

impl Default for ValidateCikFormatOutput {
    /// Returns a default output using the CIK for Berkshire Hathaway (CIK: 1067983).
    fn default() -> Self {
        Self {
            validated_cik: Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW)
                .expect("Hardcoded CIK should always be valid"),
        }
    }
}

impl fmt::Display for ValidateCikFormatOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tValid CIK: {}", self.validated_cik,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for [`ValidateCikFormatOutput`].
///
/// This struct is used to specify updates to the output data in a controlled, partial manner.
/// Fields set to `None` will not be updated. Used in conjunction with the state machine's
/// update mechanism to ensure safe and explicit state transitions.
pub struct ValidateCikFormatOutputUpdater {
    /// Optional new value for the validated CIK.
    pub cik: Option<Cik>,
}

impl ValidateCikFormatOutputUpdater {
    /// Creates a new builder for constructing [`ValidateCikFormatOutputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ValidateCikFormatOutputUpdaterBuilder {
        ValidateCikFormatOutputUpdaterBuilder::new()
    }
}

/// Updater builder for the validation output.
pub struct ValidateCikFormatOutputUpdaterBuilder {
    cik: Option<Cik>,
}

/// Builder for [`ValidateCikFormatOutputUpdater`].
///
/// This builder allows for ergonomic and explicit construction of updater instances,
/// supporting method chaining and optional fields. Use `.build()` to produce the updater.
impl ValidateCikFormatOutputUpdaterBuilder {
    /// Creates a new updater builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self { cik: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    /// Sets the CIK for the updater.
    ///
    /// # Panics
    ///
    /// Panics if the CIK is not valid.
    pub fn cik(mut self, cik: impl Into<String>) -> Self {
        self.cik = Some(Cik::new(&cik.into()).expect("Provided CIK string must be valid format"));
        self
    }

    /// Builds the updater instance from the builder.
    #[must_use]
    pub fn build(self) -> ValidateCikFormatOutputUpdater {
        ValidateCikFormatOutputUpdater { cik: self.cik }
    }
}

impl Default for ValidateCikFormatOutputUpdaterBuilder {
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
        BERKSHIRE_HATHAWAY_CIK_RAW, Cik, ValidateCikFormatOutput,
        ValidateCikFormatOutputUpdaterBuilder,
    };
    use crate::traits::state_machine::state::StateData;
    use state_maschine::prelude::StateData as SMStateData;

    #[test]
    fn should_return_reference_to_default_validation_state_data_when_initialized_with_default() {
        let validation_state_data = ValidateCikFormatOutput::default();

        let expected_result = &ValidateCikFormatOutput::default();

        let result = validation_state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let validation_state_data = &ValidateCikFormatOutput::new("12345")
            .expect("Provided hardcoded CIK should always be valid");

        let expected_result = &ValidateCikFormatOutput::default();

        let result = validation_state_data.state();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_specified_string_when_update_contains_specified_string() {
        let mut state_data = ValidateCikFormatOutput::default();
        let update = ValidateCikFormatOutputUpdaterBuilder::default()
            .cik("12345")
            .build();

        let expected_result = &ValidateCikFormatOutput::new("0000012345")
            .expect("Provided hardcoded CIK should always be valid");

        StateData::update_state(&mut state_data, update)
            .expect("Provided hardcoded update should succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut state_data = ValidateCikFormatOutput::default();
        let update = ValidateCikFormatOutputUpdaterBuilder::default()
            .cik("12345")
            .cik("067890")
            .build();

        let expected_result = &ValidateCikFormatOutput::new("0067890")
            .expect("Provided hardcoded CIK should always be valid");

        StateData::update_state(&mut state_data, update)
            .expect("Provided hardcoded update should succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = ValidateCikFormatOutput::default();
        let empty_update = ValidateCikFormatOutputUpdaterBuilder::default().build();

        let expected_result = &ValidateCikFormatOutput::default();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Provided hardcoded update should succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_formatted_and_validated_default_cik_string_when_validation_output_data_initialized_with_default()
     {
        let validation_state_data = &ValidateCikFormatOutput::default();
        let formatted_and_validated_berkshire_cik = Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW)
            .expect("Provided hardcoded CIK should always be valid");

        let expected_result = formatted_and_validated_berkshire_cik.value();

        let result = validation_state_data.state().cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic]
    fn should_panic_when_comparing_valid_but_unformatted_default_cik_with_formatted_and_validated_default_output()
     {
        let validation_state_data = &ValidateCikFormatOutput::default();
        let expected_result = BERKSHIRE_HATHAWAY_CIK_RAW;

        let result = validation_state_data.state().cik();

        assert_eq!(result, expected_result);
    }
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_output_data_trait() {
        implements_auto_traits::<ValidateCikFormatOutput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_output_data_trait() {
        implements_send::<ValidateCikFormatOutput>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_output_data_trait() {
        implements_sync::<ValidateCikFormatOutput>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_output_data_trait() {
        implements_send::<ValidateCikFormatOutput>();
        implements_sync::<ValidateCikFormatOutput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_output_data_trait() {
        implements_sized::<ValidateCikFormatOutput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_output_data_trait() {
        implements_hash::<ValidateCikFormatOutput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_output_data_trait() {
        implements_partial_eq::<ValidateCikFormatOutput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_output_data_trait() {
        implements_eq::<ValidateCikFormatOutput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_output_data_trait() {
        implements_partial_ord::<ValidateCikFormatOutput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_output_data_trait() {
        implements_ord::<ValidateCikFormatOutput>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_output_data_trait() {
        implements_default::<ValidateCikFormatOutput>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_output_data_trait() {
        implements_debug::<ValidateCikFormatOutput>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_output_data_trait() {
        implements_clone::<ValidateCikFormatOutput>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_output_data_trait() {
        implements_unpin::<ValidateCikFormatOutput>();
    }
}
