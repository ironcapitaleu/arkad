/// Error module for state machine operations.
pub mod state_machine;

pub use state_machine::StateMachine;
pub use state_machine::state::State;
pub use state_machine::transition::Transition;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// State machine related error.
    StateMachine(StateMachine),
    DowncastError,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StateMachine(state_machine) => {
                write!(
                    f,
                    "Problem occured during state machine execution: '{state_machine}'"
                )
            }
            Self::DowncastError => {
                write!(
                    f,
                    "Failed to downcast error of type `ErrorKind` into more specific error."
                )
            }
        }
    }
}

impl std::error::Error for ErrorKind {}

impl From<StateMachine> for ErrorKind {
    /// Converts a `StateMachine` error into an `ErrorKind`.
    fn from(error: StateMachine) -> Self {
        Self::StateMachine(error)
    }
}

impl TryInto<StateMachine> for ErrorKind {
    type Error = Self;

    /// Tries to convert an `ErrorKind` into a `StateMachine` error, if possible. Returns a `DowncastError` otherwise.
    fn try_into(self) -> Result<StateMachine, Self::Error> {
        match self {
            Self::StateMachine(state_machine) => Ok(state_machine),
            Self::DowncastError => Err(self),
        }
    }
}

impl TryInto<State> for ErrorKind {
    type Error = Self;

    /// Tries to convert an `ErrorKind` into a `State` error, if possible. Returns a `DowncastError` otherwise.
    fn try_into(self) -> Result<State, Self::Error> {
        match self {
            Self::StateMachine(sm) => match sm {
                StateMachine::State(state) => Ok(state),
                StateMachine::Transition(_) | StateMachine::InvalidConfiguration => {
                    Err(Self::DowncastError)
                }
            },
            Self::DowncastError => Err(Self::DowncastError),
        }
    }
}

impl TryInto<Transition> for ErrorKind {
    type Error = Self;

    /// Tries to convert an `ErrorKind` into a `Transition` error, if possible. Returns a `DowncastError` otherwise.
    fn try_into(self) -> Result<Transition, Self::Error> {
        match self {
            Self::StateMachine(sm) => match sm {
                StateMachine::Transition(transition) => Ok(transition),
                StateMachine::State(_) | StateMachine::InvalidConfiguration => {
                    Err(Self::DowncastError)
                }
            },
            Self::DowncastError => Err(Self::DowncastError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_be_able_to_create_state_machine_error_when_casting_from_specific_error_kind() {
        let _result: StateMachine = ErrorKind::StateMachine(StateMachine::InvalidConfiguration)
            .try_into()
            .expect(
                "Should always be able to cast harcdcoded `ErrorKind` into `StateMachine` error.",
            );
    }

    #[test]
    fn should_be_able_to_create_state_error_when_casting_from_specific_error_kind_that_is_a_state()
    {
        let _result: State = ErrorKind::StateMachine(StateMachine::State(State::InvalidCikFormat))
            .try_into()
            .expect(
                "Should always be able to cast provided harcdcoded `ErrorKind` into `State` error.",
            );
    }

    #[test]
    fn should_be_able_to_create_transition_error_when_casting_from_specific_error_kind_that_is_a_transition()
     {
        let _result: Transition =
            ErrorKind::StateMachine(StateMachine::Transition(Transition::FailedOutputConversion))
                .try_into()
                .expect(
                    "Should always be able to cast provided harcdcoded `ErrorKind` into `Transition` error.",
                );
    }

    #[test]
    fn should_be_able_to_create_state_machine_error_when_using_enum_directly() {
        let _result = StateMachine::InvalidConfiguration;
    }

    #[test]
    fn should_be_able_to_cast_into_equivalent_errorkind_error_when_having_a_statemachine_error() {
        let expected_result = ErrorKind::StateMachine(StateMachine::InvalidConfiguration);

        let result: ErrorKind = StateMachine::InvalidConfiguration.into();

        assert_eq!(result, expected_result);
    }
}
