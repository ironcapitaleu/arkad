//! # Transition Trait
//!
//! Provides the [`Transition`] trait for moving a state machine from one state to another.

use crate::state_machine::{StateMachine, state::State};

/// A transition of a [`StateMachine`] from state `T` to state `U`.
///
/// Implemented on a machine that is in state `T` to produce a machine in state `U`, encoding which
/// moves are valid in the type system. Consumes the machine so the old state cannot be reused.
///
/// # Associated Types
///
/// - `NewStateMachine`: The machine after the move. Must implement [`StateMachine<U>`](StateMachine).
///
/// # Type Parameters
///
/// - `T`: The source state. Must implement [`State`].
/// - `U`: The target state. Must implement [`State`].
pub trait Transition<T: State, U: State>: StateMachine<T> {
    /// The machine after transitioning to state `U`.
    type NewStateMachine: StateMachine<U>;

    /// Consumes the machine and transitions it from state `T` to state `U`.
    ///
    /// # Errors
    ///
    /// Returns an error message if the move fails, e.g. the transition is undefined or the source
    /// state's data cannot produce the target state.
    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::transition::Transition;
    use crate::tests::fixtures::{ComplexStateMachine, FirstState, SecondState};
    use pretty_assertions::assert_eq;

    #[test]
    fn should_transition_to_second_state_when_in_first_state() {
        let complex_state_machine = ComplexStateMachine::new();

        let expected_result = String::from("Second State");

        let result =
            Transition::<FirstState, SecondState>::transition_to_next_state(complex_state_machine)
                .expect("Should not fail the transitions to 'SecondState'.")
                .current_state()
                .state_name()
                .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_transition_to_first_state_when_in_first_state() {
        let complex_state_machine = ComplexStateMachine::new();

        let expected_result = String::from("First State");

        let result =
            Transition::<FirstState, FirstState>::transition_to_next_state(complex_state_machine)
                .expect("Should not fail the transitions to 'FirstState'.")
                .current_state()
                .state_name()
                .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_able_to_transition_multiple_times_state_when_transition_sequence_is_valid() {
        let complex_state_machine = ComplexStateMachine::new();

        let expected_result = String::from("Second State");

        let first_transition_result =
            Transition::<FirstState, FirstState>::transition_to_next_state(complex_state_machine)
                .expect("Should not fail the transitions to 'FirstState'.");
        let result = Transition::<FirstState, SecondState>::transition_to_next_state(
            first_transition_result,
        )
        .expect("Should not fail the transitions to 'SecondState'.")
        .current_state()
        .state_name()
        .to_string();

        assert_eq!(result, expected_result);
    }
}
