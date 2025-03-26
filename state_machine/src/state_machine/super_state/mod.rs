use super::{StateMachine, state::State};

/// The `SuperState` trait represents a state that can encapsulate and manage other states within a state machine.
///
/// This trait is used for hierarchical state machines, where a `SuperState` acts as a higher-level state
/// that contains and manages multiple sub-states. By implementing both the `State` and `StateMachine` traits,
/// `SuperState` allows complex state transitions and behavior encapsulation, enabling structured and scalable
/// state machine designs.
///
/// # Type Parameters
///
/// - `S`: A type that implements the `State` trait. This parameter specifies the type of sub-states
///   that the `SuperState` can manage. The `SuperState` itself must implement the `StateMachine<S>`
///   trait, which provides the necessary methods for managing these sub-states.
///
/// # Requirements
///
/// - Implementations of the `SuperState` trait must also implement the `StateMachine<S>` trait for the
///   specified sub-state type `S`. This ensures that the `SuperState` has the necessary functionality
///   to transition between and manage its sub-states.
///
/// # Usage
///
/// `SuperState` is typically used in scenarios where state hierarchy is necessary, such as nested state
/// machines or complex workflows where a single state might need to manage multiple sub-states internally.
/// By using the `SuperState` trait, developers can create modular and reusable state machine components
/// that encapsulate specific behaviors and transitions.
pub trait SuperState<S: State>: StateMachine<S> + State {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::transition::Transition;
    use crate::tests::common::HierarchicalStateMachine;

    #[test]
    fn should_transition_to_second_inner_state_when_in_first_inner_state_of_sample_super_state() {
        let hierarchical_state_machine = HierarchicalStateMachine::new();

        let expected_result = String::from("Second Inner State");

        let result = hierarchical_state_machine
            .transition_to_next_state()
            .expect("Should not fail inner super state transition to 'SecondInnerState'")
            .get_current_state()
            .get_current_state()
            .get_state_name()
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
        .get_current_state()
        .get_state_name()
        .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_super_state_name_when_calling_get_state_name_from_super_state() {
        let hierarchical_state_machine = HierarchicalStateMachine::new();

        let expected_result = String::from("Super State");

        let result = hierarchical_state_machine
            .get_current_state()
            .get_state_name()
            .to_string();

        assert_eq!(result, expected_result);
    }
}
