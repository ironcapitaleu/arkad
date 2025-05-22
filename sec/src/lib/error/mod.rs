pub mod state_machine;
pub use state_machine::StateMachine;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// State machine related error.
    StateMachine,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StateMachine => {
                write!(f, "Problem with the state machine.")
            }
        }
    }
}

impl std::error::Error for ErrorKind {}
