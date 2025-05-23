pub mod state;
pub mod transition;

pub use state::State;
pub use transition::Transition;

use super::ErrorKind::{self, DowncastNotPossible};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateMachine {
    /// Invalid configuration of the state machine.
    InvalidConfiguration,

    /// State-internal error.
    State(State),

    /// Transtion related error.
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
    /// Converts a `State` error into a more generic `StateMachine` error.
    fn from(error: State) -> Self {
        Self::State(error)
    }
}

impl From<Transition> for StateMachine {
    /// Converts a `Transition` error into a more generic `StateMachine` error.
    fn from(error: Transition) -> Self {
        Self::Transition(error)
    }
}

impl TryInto<State> for StateMachine {
    type Error = ErrorKind;

    /// Tries to convert a `StateMachine` into a `State` error, if possible. Returns a `DowncastNotPossible` error otherwise.
    fn try_into(self) -> Result<State, Self::Error> {
        match self {
            Self::State(state) => Ok(state),
            _ => Err(DowncastNotPossible),
        }
    }
}

impl TryInto<Transition> for StateMachine {
    type Error = ErrorKind;

    /// Tries to convert a `StateMachine` into a `Transition` error, if possible. Returns a `DowncastNotPossible` error otherwise.
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

    #[test]
    fn should_be_able_to_create_state_error_when_casting_from_specific_state_machine() {
        let _result: State = StateMachine::State(State::InvalidCikFormat)
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
        let expected_result = StateMachine::State(State::InvalidCikFormat);

        let result: StateMachine = State::InvalidCikFormat.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_able_to_cast_into_equivalent_statemachine_error_when_having_a_transition_error() {
        let expected_result = StateMachine::Transition(Transition::FailedOutputConversion);

        let result: StateMachine = Transition::FailedOutputConversion.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_able_to_cast_into_equivalent_transition_error_when_having_a_statemachine_error() {
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
