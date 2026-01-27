//! # SEC State Machine Traits
//!
//! This module defines the core traits for SEC-specific state types, as well as their associated context and state data.
//! For transition and super state traits, see the [`crate::traits::state_machine::transition`] and [`crate::traits::state_machine::super_state`] modules.
//! It extends the generic [`state_maschine`] framework with domain-specific abstractions for robust, type-safe, and testable workflows
//! in SEC data processing pipelines. Notably, it provides error handling capabilities and the ability to compute output data asynchronously.
//!
//! ## Modules
//! - [`context_data`]: Traits for defining context used within SEC state machines.
//! - [`state_data`]: Traits for defining state data used within SEC state machines.
//!
//! ## Usage
//! Implement the [`StateMachine`](super::StateMachine) trait for your SEC-specific state machine types to leverage the extensible framework
//! and integrate with concrete state, context, and data implementations found in [`crate::implementations`].
//!
//! See the documentation for each submodule for details on trait requirements and usage patterns.

use std::fmt::Display;

use async_trait::async_trait;

use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;

pub mod context_data;
pub mod state_data;

pub use context_data::Context;
pub use state_data::StateData;

/// Trait for SEC-specific states, extending the generic state machine state with domain error handling and asynchronous output data computation.
///
/// Implement this trait for SEC state types to provide custom asynchronous output computation logic with error propagation.
///
/// # Errors
///
/// Returns an error convertible into a [`StateError`] if output data computation fails.
#[async_trait]
pub trait State: SMState + Display {
    /// Computes the output data for the SEC state.
    ///
    /// # Errors
    ///
    /// Returns an error convertible into a `StateError` if the output data computation fails.
    async fn compute_output_data_async(&mut self) -> Result<(), impl Into<StateError>>;
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use tokio;

    use super::*;
    use crate::tests::common::sample_sec_state::{
        SampleSecState, SampleSecStateContext, SampleSecStateInput, SampleSecStateOutput,
    };

    #[test]
    fn should_return_name_of_sample_state_when_in_sample_state() {
        let sample_state = SampleSecState::default();

        let expected_result = String::from("Sample SEC State");

        let result = sample_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_struct_as_input_data_when_in_initial_sample_state() {
        let sample_state = SampleSecState::default();

        let expected_result = &SampleSecStateInput::default();

        let result = sample_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let sample_state = SampleSecState::default();

        let _result = sample_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let sample_state = SampleSecState::default();

        let expected_result = false;

        let result = sample_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let sample_state = SampleSecState::default();

        let expected_result = &SampleSecStateContext::default();

        let result = sample_state.context_data();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_trait() {
        implements_auto_traits::<SampleSecState>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_trait() {
        implements_send::<SampleSecState>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_trait() {
        implements_sync::<SampleSecState>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_trait() {
        implements_send::<SampleSecState>();
        implements_sync::<SampleSecState>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_trait() {
        implements_sized::<SampleSecState>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_trait() {
        implements_hash::<SampleSecState>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_trait() {
        implements_partial_eq::<SampleSecState>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_trait() {
        implements_eq::<SampleSecState>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_trait() {
        implements_partial_ord::<SampleSecState>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_trait() {
        implements_ord::<SampleSecState>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_state_trait() {
        implements_default::<SampleSecState>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_trait() {
        implements_debug::<SampleSecState>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_trait() {
        implements_clone::<SampleSecState>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_trait() {
        implements_unpin::<SampleSecState>();
    }

    #[test]
    fn should_return_default_context_data_when_called_with_state_reference() {
        let sample_state = &SampleSecState::default();
        let ref_to_sample_state = &SampleSecState::default();

        let expected_result = sample_state.context_data();

        let result = ref_to_sample_state.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_sample_state = &mut SampleSecState::default();

        let expected_result = false;

        let result = ref_to_sample_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_reference_state()
     {
        let ref_to_sample_state = &SampleSecState::default();

        let _result = ref_to_sample_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_name_of_sample_state_when_calling_reference_to_sample_state() {
        let ref_to_sample_state = &SampleSecState::default();

        let expected_result = String::from("Sample SEC State");

        let result = ref_to_sample_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_as_input_data_when_reference_sample_state_in_initial_state()
    {
        let ref_to_sample_state = &SampleSecState::default();

        let expected_result = &SampleSecStateInput::default();

        let result = ref_to_sample_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_change_input_data_when_computing_output_data() {
        let mut sample_state = SampleSecState::default();

        let expected_result = &sample_state.input_data().clone();

        sample_state
            .compute_output_data_async()
            .await
            .expect("Default test state should always compute output successfully");
        let result = sample_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_correct_output_data_when_computing_output_data() {
        let mut sample_state = SampleSecState::default();

        let expected_result = &SampleSecStateOutput::default();

        sample_state
            .compute_output_data_async()
            .await
            .expect("Default test state should always compute output successfully");

        let result = sample_state.output_data().unwrap();

        assert_eq!(result, expected_result);
    }
}
