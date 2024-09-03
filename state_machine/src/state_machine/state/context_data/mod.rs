use std::{fmt::Debug, hash::Hash};

/// The `ContextData` trait defines the behavior and characteristics of context data within a state machine.
///
/// This trait specifies how context data should be accessed and updated within a state. Context data represents
/// additional information or environmental settings that are relevant to the state machine's operation. It can be
/// used to store global or shared state information that can be accessed and modified by various states.
///
/// # Associated Types
///
/// - `UpdateType`: Represents the type of updates that can be applied to the context data. This type defines how
///   context data can be modified or refreshed, allowing for flexible updates based on specific requirements.
///
/// # Required Traits
///
/// Implementations of the `ContextData` trait must also implement several Rust standard traits to ensure
/// thread safety, comparison, and debugging capabilities:
/// - `Debug`: Allows the context data to be formatted using the `{:?}` formatter, which is useful for debugging.
/// - `Send`, `Sync`, `Unpin`: Ensure that the context data can be safely transferred and accessed across threads.
/// - `Clone`, `PartialEq`, `PartialOrd`, `Hash`, `Eq`, `Ord`: Support comparison and hashing, which is
///   necessary for certain data structures like sets or maps.
///
/// # Methods
///
/// The `ContextData` trait defines two key methods:
///
/// - `get_context`: Returns a reference to the context data. This method provides access to the current state of the context data.
/// - `update_context`: Updates the context data based on the provided updates. This method allows modifications to the context data,
///   applying the changes specified by the `UpdateType`.
pub trait ContextData:
    Debug + Send + Sync + Unpin + Clone + PartialEq + PartialOrd + Hash + Eq + Ord
{
    type UpdateType;

    /// Returns a reference to the context data.
    ///
    /// This method provides access to the current context data, allowing other parts of the state machine
    /// to read the context information. It returns a reference to the data, ensuring that the actual context
    /// is not modified by this method.
    ///
    /// # Returns
    ///
    /// A reference to the context data of the implementing type.
    fn get_context(&self) -> &Self;

    /// Updates the context data based on the provided updates.
    ///
    /// This method allows the context data to be modified according to the changes specified in the
    /// `updates` parameter. The type of `updates` is defined by the `UpdateType` associated type,
    /// which allows for flexible and specific update mechanisms tailored to the context's needs.
    ///
    /// # Parameters
    ///
    /// - `updates`: A value of type `UpdateType` that contains the changes to be applied to the context data.
    fn update_context(&mut self, updates: Self::UpdateType);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::{SampleStateContext, SampleStateContextUpdaterBuilder};

    #[test]
    fn should_return_reference_to_default_sample_context_when_initialized_with_default() {
        let sample_context = &SampleStateContext::default();

        let expected_result = &SampleStateContext::default();

        let result = sample_context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_new_as_constructor() {
        let sample_context = &SampleStateContext::new(String::from("Demir ist der Boss."));

        let default_sample_context = &SampleStateContext::default();

        let result = sample_context.get_context();

        assert_ne!(result, default_sample_context);
    }

    #[test]
    fn should_update_context_data_to_specified_string_when_update_contains_specified_string() {
        let mut context = SampleStateContext::default();
        let update = SampleStateContextUpdaterBuilder::default()
            .context_data(String::from("Updated Context!"))
            .build();

        let expected_result = &SampleStateContext::new(String::from("Updated Context!"));

        context.update_context(update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_context_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut context = SampleStateContext::default();
        let update = SampleStateContextUpdaterBuilder::default()
            .context_data(String::from("First Update!"))
            .context_data(String::from("Latest Update!"))
            .build();

        let expected_result = &SampleStateContext::new(String::from("Latest Update!"));

        context.update_context(update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = SampleStateContext::default();
        let empty_update = SampleStateContextUpdaterBuilder::default().build();

        let expected_result = &SampleStateContext::default();

        context.update_context(empty_update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }
}
