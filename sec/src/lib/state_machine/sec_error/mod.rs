/// Error type for the SEC state machine and all future states.
///
/// This enum is designed to be easily extended as new states and error types are added.
/// Use specific variants for each error domain, and wrap underlying errors as needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecError {
    /// Error for invalid CIK format.
    InvalidCikFormat(String),

    /// Error for invalid input data in a state.
    InvalidInputData { state: String, reason: String },

    /// Error for failed state transition.
    StateTransitionError {
        from: String,
        to: String,
        reason: String,
    },

    /// Error for output data computation failure.
    OutputComputationError { state: String, reason: String },

    /// Wrapper for other error types (for extensibility).
    Other {
        state: Option<String>,
        source: String,
    },
}

impl std::fmt::Display for SecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCikFormat(invalid_cik_string) => {
                write!(f, "Invalid CIK format: {invalid_cik_string}")
            }
            Self::InvalidInputData { state, reason } => {
                write!(f, "Invalid input data in state '{state}': {reason}")
            }
            Self::StateTransitionError { from, to, reason } => {
                write!(f, "Failed to transition from '{from}' to '{to}': {reason}")
            }
            Self::OutputComputationError { state, reason } => {
                write!(f, "Output computation error in state '{state}': {reason}")
            }
            Self::Other { state, source } => {
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

#[cfg(test)]
mod tests {
    use super::SecError;
    use pretty_assertions::assert_eq;

    /// Arrange, Define, Act, Assert pattern is used as per project guidelines.
    #[test]
    fn should_display_invalid_cik_format_when_invalid_cik_is_provided() {
        let invalid_cik = "some_invalid_cik".to_string();

        let expected_result = "Invalid CIK format: some_invalid_cik".to_string();

        let result = format!("{}", SecError::InvalidCikFormat(invalid_cik));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_invalid_input_data_when_state_and_reason_are_provided() {
        // Arrange
        let state = "TestState".to_string();
        let reason = "Missing field".to_string();

        // Define
        let expected_result = "Invalid input data in state 'TestState': Missing field".to_string();

        // Act
        let result = format!(
            "{}",
            SecError::InvalidInputData {
                state: state,
                reason: reason,
            }
        );

        // Assert
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_state_transition_error_when_from_to_and_reason_are_provided() {
        // Arrange
        let from = "StateA".to_string();
        let to = "StateB".to_string();
        let reason = "Not allowed".to_string();

        // Define
        let expected_result =
            "Failed to transition from 'StateA' to 'StateB': Not allowed".to_string();

        // Act
        let result = format!(
            "{}",
            SecError::StateTransitionError {
                from: from,
                to: to,
                reason: reason,
            }
        );

        // Assert
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_output_computation_error_when_state_and_reason_are_provided() {
        // Arrange
        let state = "ComputeState".to_string();
        let reason = "Overflow".to_string();

        // Define
        let expected_result =
            "Output computation error in state 'ComputeState': Overflow".to_string();

        // Act
        let result = format!(
            "{}",
            SecError::OutputComputationError {
                state: state.clone(),
                reason: reason.clone(),
            }
        );

        // Assert
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_other_error_with_state_when_state_and_source_are_provided() {
        // Arrange
        let state = Some("OtherState".to_string());
        let source = "Some error".to_string();

        // Define
        let expected_result = "Other error in state 'OtherState': Some error".to_string();

        // Act
        let result = format!(
            "{}",
            SecError::Other {
                state: state.clone(),
                source: source.clone(),
            }
        );

        // Assert
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_other_error_without_state_when_only_source_is_provided() {
        // Arrange
        let state = None;
        let source = "Unknown error".to_string();

        // Define
        let expected_result = "Other error: Unknown error".to_string();

        // Act
        let result = format!(
            "{}",
            SecError::Other {
                state,
                source: source.clone(),
            }
        );

        // Assert
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_implement_error_trait_for_sec_error() {
        // Arrange
        let err = SecError::InvalidCikFormat("bad_cik".to_string());

        // Define
        let expected_result = "Invalid CIK format: bad_cik".to_string();

        // Act
        let err_ref: &dyn std::error::Error = &err;
        let result = err_ref.to_string();

        // Assert
        assert_eq!(result, expected_result);
    }
}
