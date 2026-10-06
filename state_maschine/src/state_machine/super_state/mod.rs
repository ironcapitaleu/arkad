//! # Super-State Trait
//!
//! Provides the [`SuperState`] trait for hierarchical states: a state that is itself a state
//! machine.
//!
//! This is what lets a group of states appear as a single state to an outer machine while
//! internally driving its own sub-states.

use super::{StateMachine, state::State};

/// A state that is itself a state machine over sub-states.
///
/// Bundles the [`State`] and [`StateMachine`] bounds so a super-state can both participate as one
/// state in a parent machine and drive its own inner states.
///
/// # Type Parameters
///
/// - `S`: The active inner state type. Must implement [`State`].
pub trait SuperState<S: State>: StateMachine<S> + State {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::transition::Transition;
    use crate::tests::fixtures::HierarchicalStateMachine;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_transition_to_second_inner_state_when_in_first_inner_state_of_sample_super_state() {
        let hierarchical_state_machine = HierarchicalStateMachine::new();

        let expected_result = String::from("Second Inner State");

        let result = hierarchical_state_machine
            .transition_to_next_state()
            .expect("Should not fail inner super state transition to 'SecondInnerState'")
            .current_state()
            .current_state()
            .state_name()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_transition_to_outer_sample_state_when_in_second_inner_state_of_sample_super_state() {
        let hierarchical_state_machine = HierarchicalStateMachine::new();

        let expected_result = String::from("Sample State");

        let hierarchical_state_machine = hierarchical_state_machine
            .transition_to_next_state()
            .expect("Should not fail inner super state transition to 'SecondInnerState");
        let result = hierarchical_state_machine.transition_to_next_state()
        .expect("Should not fail transition from inner state 'SecondInnerState' to outer state 'SampleState'")
        .current_state()
        .state_name()
        .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_super_state_name_when_calling_state_name_from_super_state() {
        let hierarchical_state_machine = HierarchicalStateMachine::new();

        let expected_result = String::from("Super State");

        let result = hierarchical_state_machine
            .current_state()
            .state_name()
            .to_string();

        assert_eq!(result, expected_result);
    }
}
