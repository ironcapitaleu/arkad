//! # State Data Trait
//!
//! Provides the [`StateData`] trait for an SEC state's input/output data, adding fallible,
//! domain-typed updates to the generic [`state_maschine`] state data.
//!
//! State data is the per-computation data a state reads and produces, as opposed to the ambient
//! [`Context`](super::Context). SEC updates can fail (an update may itself need validation), so
//! the trait makes updating return a [`StateError`] rather than the framework's infallible update.

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;

/// An SEC state's input/output data, supporting fallible updates.
///
/// Refines the generic [`SMStateData`] so updates return a [`StateError`] instead of being
/// infallible, keeping error handling uniform across the pipeline. Implemented by every state's
/// input and output types.
pub trait StateData: SMStateData {
    /// Applies `updates` to the state data.
    ///
    /// # Errors
    ///
    /// Returns a [`StateError`] if the update is rejected (e.g. it fails validation).
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError>;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::StateData as SMStateData;

    // For this case, using the `SampleSecStateInput` as a concrete implementation of `StateData`.
    use crate::tests::fixtures::sample_sec_state::data::input::{
        SampleSecStateInput, SampleSecStateInputUpdaterBuilder,
    };
    use crate::traits::state_machine::state::StateData;

    #[test]
    fn should_return_reference_to_default_sample_state_data_when_initialized_with_default() {
        let default_sample_state_data = SampleSecStateInput::default();

        let expected_result = &SampleSecStateInput::default();

        let result = default_sample_state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let sample_state_data = &SampleSecStateInput::new("0000000000");

        let default_sample_state_data = &SampleSecStateInput::default();

        let result = sample_state_data.state();

        assert_ne!(result, default_sample_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_string_when_update_contains_specified_string() {
        let mut state_data = SampleSecStateInput::default();
        let update = SampleSecStateInputUpdaterBuilder::default()
            .input_data("12345")
            .build();

        let expected_result = &SampleSecStateInput::new("12345");

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");

        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut state_data = SampleSecStateInput::default();
        let update = SampleSecStateInputUpdaterBuilder::default()
            .input_data("1234567890")
            .input_data("0000000000")
            .build();

        let expected_result = &SampleSecStateInput::new("0000000000");

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = SampleSecStateInput::default();
        let empty_update = SampleSecStateInputUpdaterBuilder::default().build();

        let expected_result = &SampleSecStateInput::default();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_data_when_sample_input_data_initialized_with_default() {
        let sample_state_data = &SampleSecStateInput::default();

        let expected_result = &"Hello".to_string();

        let result = sample_state_data.state().input_data();

        assert_eq!(result, expected_result);
    }
}
