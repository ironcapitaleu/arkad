//! # SEC Context Trait
//!
//! This module defines the [`Context`] trait for SEC-specific state machines, extending the generic
//! [`state_maschine::state_machine::state::ContextData`] trait with domain-specific retry logic.
//!
//! Context represents external or environmental information that may influence internal state computations
//! in the SEC state machine framework, but is usally not directly tied to or mutated by state transitions themselves.
//! Typical examples include retry policies, configuration parameters, or metadata required for workflows (e.g., time, ...).
//!
//! ## Usage
//! Implement [`Context`] for your SEC context types to enable retry logic and context management
//! during state transitions. The trait enforces a consistent interface for querying retry capabilities and limits.
//!
//! See also:
//! - [`crate::traits::state_machine::state::StateData`]: For state data management.
//! - [`crate::implementations`]: For concrete context implementations used in SEC ETL pipelines.
//! - [`crate::error`]: For error types used in context-aware operations.

use state_maschine::prelude::ContextData as SMContextData;

/// Trait for SEC-specific context, extending the generic state machine context trait with retry logic.
///
/// Implement this trait for SEC context types to provide custom retry policies and metadata.
pub trait Context: SMContextData {
    /// Returns `true` if the state can be retried, based on the maximum allowed retries.
    fn can_retry(&self) -> bool {
        self.max_retries() > 0
    }

    /// Returns the maximum number of retries allowed for the state.
    fn max_retries(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::*;

    use crate::tests::common::sample_sec_state::sec_context::{
        SampleSecStateContext, SampleSecStateContextUpdaterBuilder,
    };

    #[test]
    fn should_return_reference_to_default_sample_context_when_initialized_with_default() {
        let sample_context = SampleSecStateContext::default();

        let expected_result = &SampleSecStateContext::default();

        let result = sample_context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_new_as_constructor() {
        let sample_context = &SampleSecStateContext::new("0000000000");

        let default_sample_context = &SampleSecStateContext::default();

        let result = sample_context.get_context();

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
        let result = context.get_context();

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
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_leave_context_data_the_default_when_update_contains_a_different_string() {
        let mut context = SampleSecStateContext::default();
        let update = SampleSecStateContextUpdaterBuilder::new()
            .data("Updated Data!")
            .build();

        context.update_context(update);
        let result = context.get_context().data();

        assert_ne!(result, "Default Data");
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = SampleSecStateContext::default();
        let empty_update = SampleSecStateContextUpdaterBuilder::default().build();

        let expected_result = &SampleSecStateContext::default();

        context.update_context(empty_update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }
}
