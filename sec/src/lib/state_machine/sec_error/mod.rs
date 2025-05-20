/// Error type for the SEC state machine and all future states.
///
/// This enum is designed to be easily extended as new states and error types are added.
/// Use specific variants for each error domain, and wrap underlying errors as needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecError {
    /// Error for invalid CIK format.
    InvalidCikFormat(String),

    /// Error for invalid input data in a state.
    InvalidInputData {
        state: String,
        reason: String,
    },

    /// Error for failed state transition.
    StateTransitionError {
        from: String,
        to: String,
        reason: String,
    },

    /// Error for output data computation failure.
    OutputComputationError {
        state: String,
        reason: String,
    },

    /// Wrapper for other error types (for extensibility).
    Other {
        state: Option<String>,
        source: String,
    },
}

impl std::fmt::Display for SecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecError::InvalidCikFormat(s) => write!(f, "Invalid CIK format: {s}"),
            SecError::InvalidInputData { state, reason } => {
                write!(f, "Invalid input data in state '{state}': {reason}")
            }
            SecError::StateTransitionError { from, to, reason } => {
                write!(f, "Failed to transition from '{from}' to '{to}': {reason}")
            }
            SecError::OutputComputationError { state, reason } => {
                write!(f, "Output computation error in state '{state}': {reason}")
            }
            SecError::Other { state, source } => {
                if let Some(state) = state {
                    write!(f, "Other error in state '{state}': {source}")
                } else {
                    write!(f, "Other error: {source}")
                }
            }
        }
    }
}

impl std::error::Error for SecError {}