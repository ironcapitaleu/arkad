pub mod state;
pub mod transition;

pub use state::State;
pub use transition::Transition;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateMachine {
    /// Invalid configuration of the state machine.
    InvalidConfiguration,

    /// State-internal error.
    State,

    /// Transtion related error.
    Transition,
}

impl std::fmt::Display for StateMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::State => {
                write!(f, "Problem occured during internal state operations.")
            },
            Self::Transition => {
                write!(f, "Problem occured during state transition.")
            },
            Self::InvalidConfiguration => {
                write!(f, "Invalid configuration of the state machine.")
            },
        }
    }
}

impl std::error::Error for StateMachine {}

