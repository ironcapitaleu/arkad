//! # Validate CIK Format Input
//!
//! Provides the [`ValidateCikFormatInput`] holding the raw CIK string fed into the
//! [`ValidateCikFormat`](crate::implementations::states::extract::validate_cik_format::ValidateCikFormat)
//! state, along with its updater and builder.
//!
//! This is the *unvalidated* side of the state's data: it carries the CIK exactly as it
//! was supplied, before any trimming or zero-padding. The validated counterpart lives in
//! [`output`](super::output).
//!
//! ## See Also
//!
//! - [`output`](super::output): The validated CIK produced once this input is processed.
//! - [`crate::shared::cik`]: The CIK parsing and validation utilities applied to this input.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
/// Input data for the [`ValidateCikFormat`](super::super::ValidateCikFormat) state.
///
/// Holds the raw, unvalidated CIK string exactly as supplied, before the state trims
/// and zero-pads it. Kept distinct from the validated output type so the two forms can
/// never be confused.
pub struct ValidateCikFormatInput {
    /// The unvalidated CIK string provided for validation.
    pub raw_cik: String,
}

impl ValidateCikFormatInput {
    /// Creates input data from a raw CIK string.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::validate_cik_format::data::input::ValidateCikFormatInput;
    ///
    /// let input = ValidateCikFormatInput::new("1067983");
    /// assert_eq!(input.cik(), "1067983");
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

    /// Delegates to the SEC [`StateData::update_state`] implementation.
    ///
    /// # Panics
    /// Panics if the fallible SEC update returns an error.
    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Err(e) = <Self as StateData>::update_state(self, updates) {
            panic!("StateData::update_state failed: {e}")
        }
    }
}

impl fmt::Display for ValidateCikFormatInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tCIK: {}", self.raw_cik)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Partial update for a [`ValidateCikFormatInput`].
///
/// Fields set to `None` are left unchanged when the updater is applied, so callers can
/// modify the raw CIK without restating the rest.
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

/// Fluent builder for a [`ValidateCikFormatInputUpdater`].
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
    use state_maschine::prelude::StateData as SMStateData;

    use super::{ValidateCikFormatInput, ValidateCikFormatInputUpdaterBuilder};
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::traits::state_machine::state::StateData;

    fn test_input() -> ValidateCikFormatInput {
        ValidateCikFormatInput::new(BERKSHIRE_HATHAWAY_CIK_RAW)
    }

    #[test]
    fn should_return_reference_to_default_validation_state_data_when_initialized_with_default() {
        let default_validation_state_data = test_input();

        let expected_result = &test_input();

        let result = default_validation_state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let validation_state_data = &ValidateCikFormatInput::new("0000000000");

        let default_validation_state_data = &test_input();

        let result = validation_state_data.state();

        assert_ne!(result, default_validation_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_string_when_update_contains_specified_string() {
        let mut state_data = test_input();
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
        let mut state_data = test_input();
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
        let mut state_data = test_input();
        let empty_update = ValidateCikFormatInputUpdaterBuilder::default().build();

        let expected_result = &test_input();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_cik_when_validation_input_data_initialized_with_test_input() {
        let validation_state_data = &test_input();

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
