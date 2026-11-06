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
//! use sec::error::state_machine::{StateMachine, State};
//!
//! let _err = StateMachine::State(State::InvalidInput);
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
/// Error occurring at the state machine level.
///
/// Groups the three ways execution can fail: a misconfigured machine, a failure inside a state,
/// or a failure transitioning between states. The latter two wrap the [`State`] and [`Transition`]
/// errors respectively.
pub enum StateMachine {
    /// Invalid configuration of the state machine.
    InvalidConfiguration,

    /// A failure occurred inside a state.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    State(State),

    /// A failure occurred while transitioning between states.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    Transition(Transition),
}

impl Display for StateMachine {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::State(state) => write!(f, "{state}"),
            Self::Transition(transition) => write!(f, "{transition}"),
            Self::InvalidConfiguration => write!(
                f,
                "[InvalidConfiguration] Invalid configuration of the state machine"
            ),
        }
    }
}

impl Error for StateMachine {
    /// Forwards to the source of a wrapped [`State`] or [`Transition`] error, as
    /// `#[error(transparent)]` does.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::State(state) => state.source(),
            Self::Transition(transition) => transition.source(),
            Self::InvalidConfiguration => None,
        }
    }
}

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
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_state_machine_error() {
        implements_auto_traits::<StateMachine>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_state_machine_error() {
        implements_send::<StateMachine>();
    }

    #[test]
    const fn should_implement_sync_for_state_machine_error() {
        implements_sync::<StateMachine>();
    }

    #[test]
    const fn should_be_thread_safe_for_state_machine_error() {
        implements_send::<StateMachine>();
        implements_sync::<StateMachine>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_for_state_machine_error() {
        implements_sized::<StateMachine>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_state_machine_error() {
        implements_hash::<StateMachine>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_state_machine_error() {
        implements_partial_eq::<StateMachine>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_state_machine_error() {
        implements_eq::<StateMachine>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_state_machine_error() {
        implements_partial_ord::<StateMachine>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_state_machine_error() {
        implements_ord::<StateMachine>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_state_machine_error() {
        implements_debug::<StateMachine>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_state_machine_error() {
        implements_clone::<StateMachine>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_state_machine_error() {
        implements_unpin::<StateMachine>();
    }

    #[test]
    fn should_be_able_to_create_state_error_when_casting_from_specific_state_machine() {
        let state_machine_error = StateMachine::State(State::InvalidInput);

        let _result: State = state_machine_error
            .try_into()
            .expect("Should always be able to cast provided hardcoded `StateMachine` error into `State` error");
    }

    #[test]
    fn should_be_able_to_create_transition_error_when_casting_from_specific_state_machine() {
        let state_machine_error = StateMachine::Transition(Transition::FailedContextConversion(
            transition::FailedContextConversion::new("StateA", "StateB", "Sample reason"),
        ));

        let _result: Transition = state_machine_error.try_into().expect("Should always be able to cast provided hardcoded `StateMachine` error into `Transition` error");
    }

    #[test]
    fn should_be_able_to_create_state_machine_error_when_using_enum_directly() {
        let expected_result = StateMachine::InvalidConfiguration;

        let result = StateMachine::InvalidConfiguration;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_able_to_cast_into_equivalent_state_machine_error_when_having_a_state_error() {
        let state_error = State::InvalidInput;

        let expected_result = StateMachine::State(State::InvalidInput);

        let result: StateMachine = state_error.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_able_to_cast_into_equivalent_state_machine_error_when_having_a_transition_error() {
        let error = Transition::FailedOutputConversion(transition::FailedOutputConversion::new(
            "StateA",
            "StateB",
            "Sample reason",
        ));

        let expected_result = StateMachine::Transition(error.clone());

        let result: StateMachine = error.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "left == right")]
    fn should_be_failing_when_trying_to_compare_casting_result_from_different_enum_variant() {
        let transition_error = Transition::FailedContextConversion(
            transition::FailedContextConversion::new("StateX", "StateY", "Sample reason"),
        );

        let expected_result = StateMachine::Transition(Transition::FailedOutputConversion(
            transition::FailedOutputConversion::new("StateA", "StateB", "Sample reason"),
        ));

        let result: StateMachine = transition_error.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_only_the_inner_error_when_state_machine_wraps_a_state_error() {
        let state_machine = StateMachine::State(State::InvalidInput);

        let expected_result = "[StateError] A state level error occurred, Reason: 'Invalid input data provided to state'";

        let result = state_machine.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_only_the_inner_error_when_state_machine_wraps_a_transition_error() {
        let state_machine = StateMachine::Transition(Transition::MissingOutput(
            transition::MissingOutput::new("Validate CIK Format", "Prepare SEC Request"),
        ));

        let expected_result = "[MissingOutput] Failure during transition from 'Validate CIK Format' to 'Prepare SEC Request', Reason: 'Output data has not been computed'";

        let result = state_machine.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_start_with_a_bracketed_name_when_state_machine_is_invalid_configuration() {
        let state_machine = StateMachine::InvalidConfiguration;

        let expected_result = "[InvalidConfiguration] Invalid configuration of the state machine";

        let result = state_machine.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_forward_source_to_the_domain_error_when_state_machine_wraps_a_state_error() {
        let cik_error = crate::shared::cik::CikError::new(
            crate::shared::cik::InvalidCikReason::ContainsNonNumericCharacters,
            "12345abcde",
        );
        let error = StateMachine::State(State::InvalidCikFormat(state::InvalidCikFormat::new(
            "Validate CIK Format",
            cik_error,
        )));

        let expected_result = Some(
            "[CikError] Invalid CIK, Reason: 'CIK contains non-numeric characters', Input: '12345abcde'"
                .to_string(),
        );

        let result = Error::source(&error).map(ToString::to_string);

        assert_eq!(result, expected_result);
    }
}
