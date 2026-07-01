//! # State Machine Trait
//!
//! Provides the [`StateMachine`] trait, the framework's central abstraction, plus its submodules
//! for states, super-states, and transitions.
//!
//! ## Modules
//!
//! - [`state`]: The [`State`] trait and its context and data traits.
//! - [`super_state`]: The [`SuperState`](super_state::SuperState) trait for hierarchical states.
//! - [`transition`]: The [`Transition`](transition::Transition) trait for moving between states.

pub mod state;
pub mod super_state;
pub mod transition;

use crate::state_machine::state::State;

/// A state machine managing and advancing a current state of type `S`.
///
/// The framework's central trait: it gives access to the current state and the means to drive it
/// forward. Implementors decide what "running" and "advancing" mean for their domain.
///
/// # Type Parameters
///
/// - `S`: The state type the machine manages. Must implement [`State`].
pub trait StateMachine<S: State> {
    /// Returns a reference to the current state.
    fn current_state(&self) -> &S;

    /// Returns a mutable reference to the current state.
    fn current_state_mut(&mut self) -> &mut S;

    /// Runs the state machine, executing its state logic and transitions.
    fn run(&mut self);

    /// Advances the state machine to its next state.
    fn advance_state(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::{
        SampleState, SampleStateContext, SampleStateData, SimpleStateMachine,
    };
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_state_has_been_advanced_and_computed_the_output()
     {
        let sample_state_machine = SimpleStateMachine::default();

        let _result = sample_state_machine
            .current_state()
            .output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_true_when_state_has_advanced_and_computed_the_output() {
        let mut sample_state_machine = SimpleStateMachine::default();

        let expected_result = true;

        sample_state_machine.advance_state();

        let result = sample_state_machine
            .current_state()
            .has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_same_output_data_even_when_state_has_advanced_different_amount_of_times() {
        let mut sm1: SimpleStateMachine = SimpleStateMachine::default();
        sm1.advance_state();
        let mut sm2 = SimpleStateMachine::default();
        sm2.advance_state();
        sm2.advance_state();

        let expected_result = sm1.current_state().output_data();

        let result = sm2.current_state().output_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_true_when_state_has_advanced_multiple_times_without_transition_and_computed_the_output()
     {
        let mut sample_state_machine = SimpleStateMachine::default();

        let expected_result = true;

        sample_state_machine.advance_state();
        sample_state_machine.advance_state();
        let result = sample_state_machine
            .current_state()
            .has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_sample_state_as_current_state() {
        let sample_state_machine = SimpleStateMachine::default();

        let expected_result = &SampleState::default();

        let result = sample_state_machine.current_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_name_of_sample_state_when_state_machine_in_sample_state() {
        let sample_state_machine = SimpleStateMachine::default();

        let expected_result = String::from("Sample State");

        let result = sample_state_machine
            .current_state()
            .state_name()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_struct_as_input_data_when_output_data_has_not_been_computed_in_state()
     {
        let sample_state_machine = SimpleStateMachine::default();

        let expected_result = &SampleStateData::default();

        let result = sample_state_machine.current_state().input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let sample_state_machine = SimpleStateMachine::default();

        let _result = sample_state_machine
            .current_state()
            .output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let sample_state_machine = SimpleStateMachine::default();

        let expected_result = false;

        let result = sample_state_machine
            .current_state()
            .has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_true_when_state_has_computed_the_output() {
        let mut sample_state_machine = SimpleStateMachine::default();

        let expected_result = true;

        sample_state_machine
            .current_state_mut()
            .compute_output_data();
        let result = sample_state_machine
            .current_state()
            .has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let sample_state_machine = SimpleStateMachine::default();

        let expected_result = &SampleStateContext::default();

        let result = sample_state_machine.current_state().context_data();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_machine_trait() {
        implements_auto_traits::<SampleState>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_machine_trait() {
        implements_send::<SampleState>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_machine_trait() {
        implements_sync::<SampleState>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_machine_trait() {
        implements_send::<SampleState>();
        implements_sync::<SampleState>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_machine_trait() {
        implements_sized::<SampleState>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_machine_trait() {
        implements_hash::<SampleState>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_machine_trait() {
        implements_partial_eq::<SampleState>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_machine_trait() {
        implements_eq::<SampleState>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_machine_trait() {
        implements_partial_ord::<SampleState>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_machine_trait() {
        implements_ord::<SampleState>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_state_machine_trait() {
        implements_default::<SampleState>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_machine_trait() {
        implements_debug::<SampleState>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_machine_trait() {
        implements_clone::<SampleState>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_machine_trait() {
        implements_unpin::<SampleState>();
    }
}
