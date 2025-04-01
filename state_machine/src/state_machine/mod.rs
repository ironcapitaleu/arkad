pub mod state;
pub mod super_state;
pub mod transition;

use crate::state_machine::state::State;

/// The `StateMachine` trait defines the behavior and structure of a state machine.
///
/// This trait is used to represent a state machine that can manage and transition between different states
/// of type `S`. It provides methods for accessing the current state, running the state machine, and advancing
/// to the next state. Implementing this trait allows the creation of modular and reusable state machine components.
///
/// # Type Parameters
///
/// - `S`: A type that implements the `State` trait. This parameter specifies the type of state that the state
///   machine manages.
///
/// # Methods
///
/// - `get_current_state`: Returns a reference to the current state of the state machine.
/// - `get_current_state_mut`: Returns a mutable reference to the current state of the state machine, allowing
///   modification of the state.
/// - `run`: Executes the logic of the state machine. This method is expected to handle state transitions
///   and perform any necessary computations.
/// - `advance_state`: Advances the state machine to the next state. This method is typically used to trigger
///   state transitions based on certain conditions or events.
pub trait StateMachine<S: State> {
    /// Returns a reference to the current state of the state machine.
    ///
    /// # Returns
    ///
    /// A reference to the current state of type `S`.
    fn get_current_state(&self) -> &S;

    /// Returns a mutable reference to the current state of the state machine.
    ///
    /// # Returns
    ///
    /// A mutable reference to the current state of type `S`.
    fn get_current_state_mut(&mut self) -> &mut S;

    /// Runs the state machine, executing its logic.
    ///
    /// This method is responsible for running the state machine, handling any necessary state transitions,
    /// and executing the logic associated with each state.
    fn run(&mut self);

    /// Advances the state machine to the next state.
    ///
    /// This method is used to transition the state machine to the next state, based on predefined
    /// transition rules or conditions.
    fn advance_state(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::{
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
            .get_current_state()
            .get_output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_true_when_state_has_advanced_and_computed_the_output() {
        let mut sample_state_machine = SimpleStateMachine::default();

        let expected_result = true;

        sample_state_machine.advance_state();

        let result = sample_state_machine
            .get_current_state()
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

        let expected_result = sm1.get_current_state().get_output_data();

        let result = sm2.get_current_state().get_output_data();

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
            .get_current_state()
            .has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_sample_state_as_current_state() {
        let sample_state_machine = SimpleStateMachine::default();

        let expected_result = &SampleState::default();

        let result = sample_state_machine.get_current_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_name_of_sample_state_when_state_machine_in_sample_state() {
        let sample_state_machine = SimpleStateMachine::default();

        let expected_result = String::from("Sample State");

        let result = sample_state_machine
            .get_current_state()
            .get_state_name()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_struct_as_input_data_when_output_data_has_not_been_computed_in_state()
     {
        let sample_state_machine = SimpleStateMachine::default();

        let expected_result = &SampleStateData::default();

        let result = sample_state_machine.get_current_state().get_input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let sample_state_machine = SimpleStateMachine::default();

        let _result = sample_state_machine
            .get_current_state()
            .get_output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let sample_state_machine = SimpleStateMachine::default();

        let expected_result = false;

        let result = sample_state_machine
            .get_current_state()
            .has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_true_when_state_has_computed_the_output() {
        let mut sample_state_machine = SimpleStateMachine::default();

        let expected_result = true;

        sample_state_machine
            .get_current_state_mut()
            .compute_output_data();
        let result = sample_state_machine
            .get_current_state()
            .has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let sample_state_machine = SimpleStateMachine::default();

        let expected_result = &SampleStateContext::default();

        let result = sample_state_machine.get_current_state().get_context_data();

        assert_eq!(result, expected_result);
    }

    fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    fn should_still_implement_auto_traits_traits_when_implementing_state_machine_trait() {
        implements_auto_traits::<SampleState>();
    }

    fn implements_send<T: Send>() {}
    fn implements_sync<T: Sync>() {}

    #[test]
    fn should_implement_send_when_implementing_state_machine_trait() {
        implements_send::<SampleState>();
    }

    #[test]
    fn should_implement_sync_when_implementing_state_machine_trait() {
        implements_sync::<SampleState>();
    }

    #[test]
    fn should_be_thread_safe_when_implementing_state_machine_trait() {
        implements_send::<SampleState>();
        implements_sync::<SampleState>();
    }

    fn implements_sized<T: Sized>() {}
    #[test]
    fn should_be_sized_when_implementing_state_machine_trait() {
        implements_sized::<SampleState>();
    }

    fn implements_hash<T: Hash>() {}
    #[test]
    fn should_implement_hash_when_implementing_state_machine_trait() {
        implements_hash::<SampleState>();
    }

    fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    fn should_implement_partial_eq_when_implementing_state_machine_trait() {
        implements_partial_eq::<SampleState>();
    }

    fn implements_eq<T: Eq>() {}
    #[test]
    fn should_implement_eq_when_implementing_state_machine_trait() {
        implements_eq::<SampleState>();
    }

    fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    fn should_implement_partial_ord_when_implementing_state_machine_trait() {
        implements_partial_ord::<SampleState>();
    }

    fn implements_ord<T: Ord>() {}
    #[test]
    fn should_implement_ord_when_implementing_state_machine_trait() {
        implements_ord::<SampleState>();
    }

    fn implements_default<T: Default>() {}
    #[test]
    fn should_implement_default_when_implementing_state_machine_trait() {
        implements_default::<SampleState>()
    }

    fn implements_debug<T: Debug>() {}
    #[test]
    fn should_implement_debug_when_implementing_state_machine_trait() {
        implements_debug::<SampleState>();
    }

    fn implements_clone<T: Clone>() {}
    #[test]
    fn should_implement_clone_when_implementing_state_machine_trait() {
        implements_clone::<SampleState>();
    }

    fn implements_unpin<T: Unpin>() {}
    #[test]
    fn should_implement_unpin_when_implementing_state_machine_trait() {
        implements_unpin::<SampleState>();
    }
}
