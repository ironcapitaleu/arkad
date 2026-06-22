//! # State Machine Errors
//!
//! Provides the [`StateMachine`] error, the middle layer of the error hierarchy, covering
//! failures within states, during transitions, and from invalid configuration.
//!
//! It wraps the more specific [`State`] and [`Transition`] errors and is itself wrapped by
//! [`ErrorKind`]; the [`TryFrom`] impls recover the inner error.
//!
//! ## Modules
//!
//! - [`state`]: The [`State`] error for failures inside a state.
//! - [`transition`]: The [`Transition`] error for failures while moving between states.
//!
//! ## Usage
//!
//! ```rust
//! use sec::error::state_machine::{StateMachine, State, Transition};
//!
//! let err = StateMachine::State(State::InvalidInput);
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

use super::ErrorKind;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// An error occurring at the state machine level.
///
/// Groups the three ways execution can fail: a misconfigured machine, a failure inside a state,
/// or a failure transitioning between states. The latter two wrap the [`State`] and [`Transition`]
/// errors respectively.
pub enum StateMachine {
    /// The state machine was configured invalidly.
    InvalidConfiguration,

    /// A failure occurred inside a state.
    State(State),

    /// A failure occurred while transitioning between states.
    Transition(Transition),
}

impl Display for StateMachine {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::State(state) => {
                write!(
                    f,
                    "Problem occurred during internal state operations: '{state}'"
                )
            }
            Self::Transition(transition) => {
                write!(
                    f,
                    "Problem occurred during state transition: '{transition}'"
                )
            }
            Self::InvalidConfiguration => {
                write!(f, "Invalid configuration of the state machine")
            }
        }
    }
}

impl Error for StateMachine {}

impl From<State> for StateMachine {
    fn from(error: State) -> Self {
        Self::State(error)
    }
}

impl From<Transition> for StateMachine {
    fn from(error: Transition) -> Self {
        Self::Transition(error)
    }
}

impl TryFrom<StateMachine> for State {
    type Error = ErrorKind;

    /// # Errors
    ///
    /// Returns [`ErrorKind::DowncastNotPossible`] if the value is not a [`State`] error.
    fn try_from(value: StateMachine) -> Result<Self, Self::Error> {
        match value {
            StateMachine::State(state) => Ok(state),
            _ => Err(ErrorKind::DowncastNotPossible),
        }
    }
}

impl TryFrom<StateMachine> for Transition {
    type Error = ErrorKind;

    /// # Errors
    ///
    /// Returns [`ErrorKind::DowncastNotPossible`] if the value is not a [`Transition`] error.
    fn try_from(value: StateMachine) -> Result<Self, Self::Error> {
        match value {
            StateMachine::Transition(transition) => Ok(transition),
            _ => Err(ErrorKind::DowncastNotPossible),
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
        let _result: State = StateMachine::State(State::InvalidInput)
            .try_into()
            .expect("Should always be able to cast provided hardcoded `StateMachine` error into `State` error");
    }

    #[test]
    fn should_be_able_to_create_transition_error_when_casting_from_specific_state_machine() {
        let _result: Transition = StateMachine::Transition(Transition::FailedContextConversion(
            transition::FailedContextConversion::new("StateA", "StateB"),
        ))
        .try_into()
        .expect("Should always be able to cast provided hardcoded `StateMachine` error into `Transition` error");
    }

    #[test]
    fn should_be_able_to_create_state_machine_error_when_using_enum_directly() {
        let _result = StateMachine::InvalidConfiguration;
    }

    #[test]
    fn should_be_able_to_cast_into_equivalent_statemachine_error_when_having_a_state_error() {
        let expected_result = StateMachine::State(State::InvalidInput);

        let result: StateMachine = State::InvalidInput.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_able_to_cast_into_equivalent_statemachine_error_when_having_a_transition_error() {
        let error = Transition::FailedOutputConversion(transition::FailedOutputConversion::new(
            "StateA", "StateB",
        ));

        let expected_result = StateMachine::Transition(error.clone());

        let result: StateMachine = error.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic]
    fn should_be_failing_when_when_trying_to_compare_casting_result_from_different_enum_variant() {
        let expected_result = StateMachine::Transition(Transition::FailedOutputConversion(
            transition::FailedOutputConversion::new("StateA", "StateB"),
        ));

        let result: StateMachine = Transition::FailedContextConversion(
            transition::FailedContextConversion::new("StateX", "StateY"),
        )
        .into();

        assert_eq!(result, expected_result);
    }
}
