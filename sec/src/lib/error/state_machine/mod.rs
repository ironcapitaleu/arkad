//! # State Machine Error Types
//!
//! This module defines error types specific to the operation of SEC state machines, including errors
//! originating from states, transitions, and invalid state machine configurations. These error types
//! enable robust and granular error handling throughout the SEC state machine framework.
//!
//! ## Modules
//! - [`state`](state/mod.rs): Contains error types for state-internal failures (e.g., invalid input, context, or output computation).
//! - [`transition`](transition/mod.rs): Contains error types for failures during state transitions (e.g., failed output or context conversion).
//!
//! ## Types
//! - [`StateMachine`]: Enum representing all errors that can occur at the state machine level, including state and transition errors.
//! - [`State`] (state/mod.rs): Enum for errors that occur within a state.
//! - [`Transition`] (transition/mod.rs): Enum for errors that occur during state transitions.
//!
//! ## Usage
//! Use [`StateMachine`] as the primary error type for state machine operations. Errors can be downcast to [`State`] or [`Transition`] for more specific handling.
//!
//! ## Example
//! ```rust
//! use sec::error::state_machine::{StateMachine, State, Transition};
//! let err = StateMachine::State(State::InvalidInputData);
//! match err {
//!     StateMachine::State(state_err) => println!("State error: {state_err}"),
//!     StateMachine::Transition(trans_err) => println!("Transition error: {trans_err}"),
//!     StateMachine::InvalidConfiguration => println!("Invalid state machine configuration"),
//!     _ => println!("Other state machine error"),
//! }
//! ```

pub mod state;
pub mod transition;

pub use state::State;
pub use transition::Transition;

use super::ErrorKind::{self, DowncastNotPossible};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Represents all error kinds that can occur at the state machine level.
///
/// This enum encapsulates errors from invalid configurations, state-internal failures, and transition failures.
/// It supports conversions from [`State`] and [`Transition`] errors, and provides downcasting for granular error handling.
pub enum StateMachine {
    /// Invalid configuration of the state machine.
    InvalidConfiguration,

    /// State-internal error.
    State(State),

    /// Transition related error.
    Transition(Transition),
}

impl std::fmt::Display for StateMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::State(state) => {
                write!(
                    f,
                    "Problem occured during internal state operations: '{state}'"
                )
            }
            Self::Transition(transition) => {
                write!(f, "Problem occured during state transition: '{transition}'")
            }
            Self::InvalidConfiguration => {
                write!(f, "Invalid configuration of the state machine.")
            }
        }
    }
}

impl std::error::Error for StateMachine {}

impl From<State> for StateMachine {
    /// Converts a [`State`] error into a more general [`StateMachine`] error.
    ///
    /// This enables seamless propagation of state errors as state machine errors.
    fn from(error: State) -> Self {
        Self::State(error)
    }
}

impl From<Transition> for StateMachine {
    /// Converts a [`Transition`] error into a more general [`StateMachine`] error.
    ///
    /// This enables seamless propagation of transition errors as state machine errors.
    fn from(error: Transition) -> Self {
        Self::Transition(error)
    }
}

impl TryInto<State> for StateMachine {
    type Error = ErrorKind;

    /// Attempts to convert a [`StateMachine`] error into a [`State`] error.
    ///
    /// Returns `Ok(State)` if the variant matches, or `Err(ErrorKind::DowncastNotPossible)` otherwise.
    fn try_into(self) -> Result<State, Self::Error> {
        match self {
            Self::State(state) => Ok(state),
            _ => Err(DowncastNotPossible),
        }
    }
}

impl TryInto<Transition> for StateMachine {
    type Error = ErrorKind;

    /// Attempts to convert a [`StateMachine`] error into a [`Transition`] error.
    ///
    /// Returns `Ok(Transition)` if the variant matches, or `Err(ErrorKind::DowncastNotPossible)` otherwise.
    fn try_into(self) -> Result<Transition, Self::Error> {
        match self {
            Self::Transition(transition) => Ok(transition),
            _ => Err(DowncastNotPossible),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_statemachine() {
        implements_auto_traits::<StateMachine>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_have_implementend_send_when_using_statemachine() {
        implements_send::<StateMachine>();
    }

    #[test]
    const fn should_implement_sync_when_using_statemachine() {
        implements_sync::<StateMachine>();
    }

    #[test]
    const fn should_be_thread_safe_when_using_statemachine() {
        implements_send::<StateMachine>();
        implements_sync::<StateMachine>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_when_using_statemachine() {
        implements_sized::<StateMachine>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_when_using_statemachine() {
        implements_hash::<StateMachine>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_when_using_statemachine() {
        implements_partial_eq::<StateMachine>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_when_using_statemachine() {
        implements_eq::<StateMachine>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_when_using_statemachine() {
        implements_partial_ord::<StateMachine>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_when_using_statemachine() {
        implements_ord::<StateMachine>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_statemachine() {
        implements_debug::<StateMachine>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_when_using_statemachine() {
        implements_clone::<StateMachine>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_when_using_statemachine() {
        implements_unpin::<StateMachine>();
    }

    #[test]
    fn should_be_able_to_create_state_error_when_casting_from_specific_state_machine() {
        let _result: State = StateMachine::State(State::InvalidInputData)
            .try_into()
            .expect("Should always be able to cast provided harcdcoded `StateMachine` error into `State` error.");
    }

    #[test]
    fn should_be_able_to_create_transition_error_when_casting_from_specific_state_machine() {
        let _result: Transition = StateMachine::Transition(Transition::FailedContextConversion)
            .try_into()
            .expect("Should always be able to cast provided harcdcoded `StateMachine` error into `Transition` error.");
    }

    #[test]
    fn should_be_able_to_create_state_machine_error_when_using_enum_directly() {
        let _result = StateMachine::InvalidConfiguration;
    }

    #[test]
    fn should_be_able_to_cast_into_equivalent_statemachine_error_when_having_a_state_error() {
        let expected_result = StateMachine::State(State::InvalidInputData);

        let result: StateMachine = State::InvalidInputData.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_able_to_cast_into_equivalent_statemachine_error_when_having_a_transition_error() {
        let expected_result = StateMachine::Transition(Transition::FailedOutputConversion);

        let result: StateMachine = Transition::FailedOutputConversion.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic]
    fn should_be_failing_when_when_trying_to_compare_casting_result_from_different_enum_variant() {
        let expected_result = StateMachine::Transition(Transition::FailedOutputConversion);

        let result: StateMachine = Transition::FailedContextConversion.into();

        assert_eq!(result, expected_result);
    }
}
