use std::{fmt::Debug, hash::Hash};

/// The `StateData` trait defines the behavior and characteristics of state data within a state machine.
///
/// This trait is used to represent the data associated with a specific state in a state machine. It provides
/// methods to access and update the state data, ensuring that state transitions and updates are handled
/// consistently and correctly. The `StateData` trait requires the implementation of various Rust standard
/// traits to guarantee safety, comparability, and debugging capabilities.
///
/// # Associated Types
///
/// - `UpdateType`: Defines the type of updates that can be applied to the state data. This allows
///   the state data to be modified based on specific requirements, encapsulating the changes in
///   a structured manner.
///
/// # Required Traits
///
/// Implementations of the `StateData` trait must also implement several Rust standard traits to ensure
/// thread safety, comparison, and debugging capabilities:
/// - `Debug`: Allows the state data to be formatted using the `{:?}` formatter, which is useful for debugging.
/// - `Send`, `Sync`, `Unpin`: Ensure that the state data can be safely transferred and accessed across threads.
/// - `Clone`, `PartialEq`, `PartialOrd`, `Hash`, `Eq`, `Ord`: Support comparison and hashing, which is
///   necessary for certain data structures like sets or maps.
///
/// # Methods
///
/// The `StateData` trait defines two key methods:
///
/// - `state`: Returns a reference to the state data. This method provides access to the current state data,
///   allowing the state machine to inspect or read the data.
/// - `update_state`: Updates the state data based on the provided updates. This method allows modifications to the state data,
///   applying the changes specified by the `UpdateType`.
pub trait StateData:
    Debug + Send + Sync + Unpin + Clone + PartialEq + PartialOrd + Hash + Eq + Ord
{
    type UpdateType;

    /// Returns a reference to the internal state data.
    ///
    /// This method provides access to the current state data, allowing other parts of the state machine
    /// to read the state information. It returns a reference to the data, ensuring that the actual state
    /// is not modified by this method.
    ///
    /// # Returns
    ///
    /// A reference to the state data of the implementing type.
    fn state(&self) -> &Self;

    /// Updates the internal state data based on the provided updates.
    ///
    /// This method allows the state data to be modified according to the changes specified in the
    /// `updates` parameter. The type of `updates` is defined by the `UpdateType` associated type,
    /// which allows for flexible and specific update mechanisms tailored to the state's needs.
    ///
    /// # Parameters
    ///
    /// - `updates`: A value of type `UpdateType` that contains the changes to be applied to the state data.
    fn update_state(&mut self, updates: Self::UpdateType);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::{SampleStateData, SampleStateDataUpdaterBuilder};
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
