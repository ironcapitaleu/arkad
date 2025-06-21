//! # SEC State Data Trait
//!
//! This module defines the [`StateData`] trait for SEC-specific state machines, extending the generic
//! [`state_maschine::state_machine::state::StateData`] trait with domain-aware error handling.
//!
//! State data represents the internal, mutable data associated with a state in the SEC state machine framework.
//! Implementations of this trait are responsible for encapsulating and updating the input/output data
//! for each state, supporting robust, type-safe, and testable workflows.
//!
//! ## Usage
//! Implement [`StateData`] for your SEC state data types to enable controlled updates and error propagation
//! during state transitions. The trait enforces that all updates return a strongly-typed [`crate::error::State`]
//! error on failure, ensuring consistent error handling across the state machine.
//!
//! See also:
//! - [`crate::traits::state_machine::state::ContextData`]: For context data management.
//! - [`crate::implementations`]: For concrete state data implementations used in SEC ETL pipelines.
//! - [`crate::error`]: For error types used in update operations.

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;

/// Trait for SEC-specific state data, extending the generic state machine state data trait with domain error handling.
///
/// Implement this trait for SEC state data types to provide custom update logic with error propagation.
///
/// # Errors
///
/// Returns a [`crate::error::State`] if the update fails.
pub trait StateData: SMStateData {
    /// Updates the state with new data given in the `updates` parameter.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::error::State`] if the update fails.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError>;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::StateData as SMStateData;

    // For this case, using the `SampleSecStateInputData` as a concrete implementation of `StateData`.
    use crate::tests::common::sample_sec_state::sec_data::sec_input_data::{
        SampleSecStateInputData, SampleSecStateInputDataUpdaterBuilder,
    };

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
