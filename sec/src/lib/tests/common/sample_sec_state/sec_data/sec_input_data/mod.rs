//! # Sample SEC State Input Data
//!
//! This module defines the input data structure and updater patterns for the [`SampleSecState`](../../mod.rs) fixture.
//! It provides a simple `String`-based input to demonstrate the basic requirements for state input data.
//!
//! ## Types
//! - [`SampleSecStateInputData`]: Holds the unvalidated input string to be processed by the sample state.
//! - [`SampleSecStateInputDataUpdater`]: Updater type for modifying the input data.
//! - [`SampleSecStateInputDataUpdaterBuilder`]: Builder for constructing updater instances.
//!
//! ## Integration
//! - Implements [`StateData`](crate::traits::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`SampleSecState`](../../mod.rs) to receive input data.
//!
//! ## See Also
//! - [`sec_output_data`](super::sec_output_data): The corresponding output data structure.
//! - [`crate::traits::state_machine::state::StateData`]: Trait for state data integration.
use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Input data for the `SampleSecState` fixture.
pub struct SampleSecStateInputData {
    pub input_data: String,
}

impl SampleSecStateInputData {
    /// Creates a new instance of the input data for the sample state.
    pub fn new(input_data: &(impl ToString + ?Sized)) -> Self {
        Self {
            input_data: input_data.to_string(),
        }
    }

    /// Returns a reference to the raw input string.
    #[must_use]
    pub const fn input_data(&self) -> &String {
        &self.input_data
    }
}

impl StateData for SampleSecStateInputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(input_data) = updates.input_data {
            self.input_data = input_data;
        }
        Ok(())
    }
}

impl SMStateData for SampleSecStateInputData {
    type UpdateType = SampleSecStateInputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl Default for SampleSecStateInputData {
    fn default() -> Self {
        Self {
            input_data: "Hello".to_string(),
        }
    }
}

impl fmt::Display for SampleSecStateInputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tInput Data: {}", self.input_data,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for [`SampleSecStateInputData`].
pub struct SampleSecStateInputDataUpdater {
    pub input_data: Option<String>,
}

/// Builder for [`SampleSecStateInputDataUpdater`].
pub struct SampleSecStateInputDataUpdaterBuilder {
    input_data: Option<String>,
}
impl SampleSecStateInputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { input_data: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn input_data(mut self, input_data: &(impl ToString + ?Sized)) -> Self {
        self.input_data = Some(input_data.to_string());
        self
    }

    #[must_use]
    pub fn build(self) -> SampleSecStateInputDataUpdater {
        SampleSecStateInputDataUpdater {
            input_data: self.input_data,
        }
    }
}

impl Default for SampleSecStateInputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::StateData as SMStateData;

    use super::{SampleSecStateInputData, SampleSecStateInputDataUpdaterBuilder};
    use crate::traits::state_machine::state::StateData;

    #[test]
    fn should_return_reference_to_default_validation_state_data_when_initialized_with_default() {
        let default_validation_state_data = SampleSecStateInputData::default();

        let expected_result = &SampleSecStateInputData::default();

        let result = default_validation_state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let validation_state_data = &SampleSecStateInputData::new("0000000000");

        let default_validation_state_data = &SampleSecStateInputData::default();

        let result = validation_state_data.get_state();

        assert_ne!(result, default_validation_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_string_when_update_contains_specified_string() {
        let mut state_data = SampleSecStateInputData::default();
        let update = SampleSecStateInputDataUpdaterBuilder::default()
            .input_data("12345")
            .build();

        let expected_result = &SampleSecStateInputData::new("12345");

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed.");

        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut state_data = SampleSecStateInputData::default();
        let update = SampleSecStateInputDataUpdaterBuilder::default()
            .input_data("1234567890")
            .input_data("0000000000")
            .build();

        let expected_result = &SampleSecStateInputData::new("0000000000");

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = SampleSecStateInputData::default();
        let empty_update = SampleSecStateInputDataUpdaterBuilder::default().build();

        let expected_result = &SampleSecStateInputData::default();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Update with valid 'update' value should always succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_data_when_validation_input_data_initialized_with_default() {
        let validation_state_data = &SampleSecStateInputData::default();

        let expected_result = &"Hello".to_string();

        let result = validation_state_data.get_state().input_data();

        assert_eq!(result, expected_result);
    }
}
