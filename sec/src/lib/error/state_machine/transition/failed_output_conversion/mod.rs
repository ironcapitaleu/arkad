//! # Failed Output Conversion Transition Error
//!
//! This module defines the [`FailedOutputConversion`] error type, which represents a failure
//! to convert the output data of a source state into the input data of a destination state
//! during a state transition.

use thiserror::Error;

use super::Transition as TransitionError;

/// Error representing a failed output-to-input conversion during a state transition.
///
/// This error type captures which states were involved when the output data
/// could not be transformed into the input data required by the destination state.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[FailedOutputConversion] Failure during transition from '{source_state_name}' to '{target_state_name}', Reason: Failed to convert output data to input data"
)]
pub struct FailedOutputConversion {
    /// The name of the source state whose output could not be converted.
    pub source_state_name: String,
    /// The name of the target state whose input could not be produced.
    pub target_state_name: String,
}

impl FailedOutputConversion {
    /// Creates a new [`FailedOutputConversion`] error.
    #[must_use]
    pub fn new(source_state_name: impl Into<String>, target_state_name: impl Into<String>) -> Self {
        Self {
            source_state_name: source_state_name.into(),
            target_state_name: target_state_name.into(),
        }
    }
}

impl From<FailedOutputConversion> for TransitionError {
    fn from(error: FailedOutputConversion) -> Self {
        Self::FailedOutputConversion(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_create_error_with_state_names_when_new_is_called() {
        let expected_result = FailedOutputConversion {
            source_state_name: "StateA".to_string(),
            target_state_name: "StateB".to_string(),
        };

        let result = FailedOutputConversion::new("StateA", "StateB");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_transition_error_when_into_is_called() {
        let error = FailedOutputConversion::new("StateA", "StateB");

        let expected_result = TransitionError::FailedOutputConversion(error.clone());

        let result: TransitionError = error.into();

        assert_eq!(result, expected_result);
    }
}
