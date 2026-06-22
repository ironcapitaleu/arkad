//! # Context Trait
//!
//! Provides the [`Context`] trait for a SEC state's ambient context, adding a retry budget to the
//! generic [`state_maschine`] context.
//!
//! Context is the environmental information surrounding a state (shared client, retry policy,
//! configuration) that outlives any single input/output and is not mutated by transitions, as
//! opposed to the per-computation [`StateData`](super::StateData).

use state_maschine::prelude::Context as SMContext;

/// A SEC state's ambient context, exposing a retry budget.
///
/// Refines the generic [`SMContext`] with the retry limit SEC states consult, and provides
/// [`can_retry`](Context::can_retry) on top of it. Implemented by every state's context type.
pub trait Context: SMContext {
    /// Returns `true` if the state may still be retried, given its retry budget.
    fn can_retry(&self) -> bool {
        self.max_retries() > 0
    }

    /// Returns the maximum number of times the state may be retried.
    fn max_retries(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::*;

    use crate::tests::fixtures::sample_sec_state::context::{
        SampleSecStateContext, SampleSecStateContextUpdaterBuilder,
    };

    #[test]
    fn should_return_reference_to_default_sample_context_when_initialized_with_default() {
        let sample_context = SampleSecStateContext::default();

        let expected_result = &SampleSecStateContext::default();

        let result = sample_context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_new_as_constructor() {
        let sample_context = &SampleSecStateContext::new("0000000000");

        let default_sample_context = &SampleSecStateContext::default();

        let result = sample_context.context();

        assert_ne!(result, default_sample_context);
    }

    #[test]
    fn should_update_context_data_to_specified_string_when_update_contains_specified_string() {
        let mut context = SampleSecStateContext::default();
        let update = SampleSecStateContextUpdaterBuilder::new()
            .data("Updated Data!")
            .build();

        let expected_result = &SampleSecStateContext::new("Updated Data!");

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut context = SampleSecStateContext::default();
        let update = SampleSecStateContextUpdaterBuilder::new()
            .data("First Data Update!")
            .data("Latest Data Update!")
            .build();

        let expected_result = &SampleSecStateContext::new("Latest Data Update!");

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_leave_context_data_the_default_when_update_contains_a_different_string() {
        let mut context = SampleSecStateContext::default();
        let update = SampleSecStateContextUpdaterBuilder::new()
            .data("Updated Data!")
            .build();

        context.update_context(update);
        let result = context.context().data();

        assert_ne!(result, "Default Data");
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = SampleSecStateContext::default();
        let empty_update = SampleSecStateContextUpdaterBuilder::default().build();

        let expected_result = &SampleSecStateContext::default();

        context.update_context(empty_update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }
}
