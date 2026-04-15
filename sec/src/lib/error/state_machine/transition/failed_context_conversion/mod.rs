//! # Failed Context Conversion Transition Error
//!
//! This module defines the [`FailedContextConversion`] error type, which represents a failure
//! to convert the context of a source state into the context of a destination state
//! during a state transition.

use thiserror::Error;

use super::Transition as TransitionError;

/// Error representing a failed context conversion during a state transition.
///
/// This error type captures which states were involved when the context
/// could not be transformed into the context required by the destination state.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[FailedContextConversion] Failure during transition from '{source_state_name}' to '{target_state_name}', Reason: Failed to convert context data"
)]
pub struct FailedContextConversion {
    /// The name of the source state whose context could not be converted.
    pub source_state_name: String,
    /// The name of the target state whose context could not be produced.
    pub target_state_name: String,
}

impl FailedContextConversion {
    /// Creates a new [`FailedContextConversion`] error.
    #[must_use]
    pub fn new(source_state_name: impl Into<String>, target_state_name: impl Into<String>) -> Self {
        Self {
            source_state_name: source_state_name.into(),
            target_state_name: target_state_name.into(),
        }
    }
}

impl From<FailedContextConversion> for TransitionError {
    fn from(error: FailedContextConversion) -> Self {
        Self::FailedContextConversion(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_create_error_with_state_names_when_new_is_called() {
        let expected_result = FailedContextConversion {
            source_state_name: "StateA".to_string(),
            target_state_name: "StateB".to_string(),
        };

        let result = FailedContextConversion::new("StateA", "StateB");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_transition_error_when_into_is_called() {
        let error = FailedContextConversion::new("StateA", "StateB");

        let expected_result = TransitionError::FailedContextConversion(error.clone());

        let result: TransitionError = error.into();

        assert_eq!(result, expected_result);
    }
}
