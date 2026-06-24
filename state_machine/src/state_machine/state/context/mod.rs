//! # Context Trait
//!
//! Provides the [`Context`] trait for the ambient data a state reads, separate from its
//! input/output [`StateData`](super::StateData).

use std::{fmt::Debug, hash::Hash};

/// Ambient, partially-updatable data associated with a state.
///
/// Holds environmental or shared information relevant to a state (configuration, retry policy,
/// and the like) that is read during computation and changed only through explicit updates. The
/// supertrait bounds keep it thread-safe, comparable, and hashable like the state itself.
///
/// # Associated Types
///
/// - `UpdateType`: The partial update applied by [`update_context`](Context::update_context).
///
/// # Required Traits
///
/// Implementors must be `Debug + Send + Sync + Unpin + Clone + PartialEq + PartialOrd + Hash + Eq +
/// Ord`, matching the bounds on [`State`](super::State).
pub trait Context:
    Debug + Send + Sync + Unpin + Clone + PartialEq + PartialOrd + Hash + Eq + Ord
{
    /// The partial update applied by [`update_context`](Context::update_context).
    type UpdateType;

    /// Returns a reference to the context data.
    fn context(&self) -> &Self;

    /// Applies a partial `updates` value to the context.
    fn update_context(&mut self, updates: Self::UpdateType);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::{SampleStateContext, SampleStateContextUpdaterBuilder};
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn should_return_reference_to_default_sample_context_when_initialized_with_default() {
        let sample_context = &SampleStateContext::default();

        let expected_result = &SampleStateContext::default();

        let result = sample_context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_new_as_constructor() {
        let sample_context = &SampleStateContext::new(String::from("Demir ist der Boss."));

        let default_sample_context = &SampleStateContext::default();

        let result = sample_context.context();

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
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_context_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut context = SampleStateContext::default();
        let update = SampleStateContextUpdaterBuilder::default()
            .context_data("First Update!")
            .context_data("Latest Update!")
            .build();

        let expected_result = &SampleStateContext::new(String::from("Latest Update!"));

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = SampleStateContext::default();
        let empty_update = SampleStateContextUpdaterBuilder::default().build();

        let expected_result = &SampleStateContext::default();

        context.update_context(empty_update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }
}
