//! # State Data Trait
//!
//! Provides the [`StateData`] trait for the input and output data a state reads and produces,
//! separate from its ambient [`Context`](super::Context).

use std::{fmt::Debug, hash::Hash};

/// The input or output data of a state, supporting partial updates.
///
/// Encapsulates the data a state reads and produces, exposing read access and a structured update
/// path. The supertrait bounds keep it thread-safe, comparable, and hashable like the state that
/// owns it.
///
/// # Associated Types
///
/// - `UpdateType`: The partial update applied by [`update_state`](StateData::update_state).
///
/// # Required Traits
///
/// Implementors must be `Debug + Send + Sync + Unpin + Clone + PartialEq + PartialOrd + Hash + Eq +
/// Ord`, matching the bounds on [`State`](super::State).
pub trait StateData:
    Debug + Send + Sync + Unpin + Clone + PartialEq + PartialOrd + Hash + Eq + Ord
{
    /// The partial update applied by [`update_state`](StateData::update_state).
    type UpdateType;

    /// Returns a reference to the state data.
    fn state(&self) -> &Self;

    /// Applies a partial `updates` value to the state data.
    fn update_state(&mut self, updates: Self::UpdateType);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::{SampleStateData, SampleStateDataUpdaterBuilder};
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn should_return_reference_to_default_sample_state_data_when_initialized_with_default() {
        let sample_state_data = &SampleStateData::default();

        let expected_result = &SampleStateData::default();

        let result = sample_state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let sample_state_data = &SampleStateData::new(String::from("Demir ist der Boss."));

        let default_sample_state_data = &SampleStateData::default();

        let result = sample_state_data.state();

        assert_ne!(result, default_sample_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_string_when_update_contains_specified_string() {
        let mut state_data = SampleStateData::default();
        let update = SampleStateDataUpdaterBuilder::default()
            .state_data(String::from("Updated State!"))
            .build();

        let expected_result = &SampleStateData::new(String::from("Updated State!"));

        state_data.update_state(update);
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut state_data = SampleStateData::default();
        let update = SampleStateDataUpdaterBuilder::default()
            .state_data(String::from("First Update!"))
            .state_data(String::from("Latest Update!"))
            .build();

        let expected_result = &SampleStateData::new(String::from("Latest Update!"));

        state_data.update_state(update);
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = SampleStateData::default();
        let empty_update = SampleStateDataUpdaterBuilder::default().build();

        let expected_result = &SampleStateData::default();

        state_data.update_state(empty_update);
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }
}
