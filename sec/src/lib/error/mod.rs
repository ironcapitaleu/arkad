pub mod state_machine;
pub use state_machine::StateMachine;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// State machine related error.
    StateMachine(StateMachine),
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
        }
    }
}

impl std::error::Error for ErrorKind {}
