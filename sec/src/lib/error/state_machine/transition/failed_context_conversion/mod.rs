//! # Failed Context Conversion Error
//!
//! Provides the [`FailedContextConversion`] error: a source state's context could not be converted
//! into the destination state's context during a transition.

use thiserror::Error;

use super::Transition as TransitionError;

/// A source state's context could not be converted into the destination state's context.
///
/// Records both states involved, so the failed conversion pinpoints which transition broke.
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
    /// Creates a new error from the source and target state names.
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
