//! # Failed Context Conversion Error
//!
//! Provides the [`FailedContextConversion`] error: a source state's context could not be converted
//! into the destination state's context during a transition.

use thiserror::Error;

use super::Transition as TransitionError;

/// Error representing a failed context conversion during a state transition.
///
/// Records both states involved and the reason, so the failed conversion pinpoints which transition
/// broke and why.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[FailedContextConversion] Failure during transition from '{source_state_name}' to '{target_state_name}', Reason: '{reason}'"
)]
pub struct FailedContextConversion {
    /// The name of the source state whose context could not be converted.
    pub source_state_name: String,
    /// The name of the target state whose context could not be produced.
    pub target_state_name: String,
    /// Why the conversion failed.
    pub reason: String,
}

impl FailedContextConversion {
    /// Creates a new [`FailedContextConversion`] error.
    #[must_use]
    pub fn new(
        source_state_name: impl Into<String>,
        target_state_name: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            source_state_name: source_state_name.into(),
            target_state_name: target_state_name.into(),
            reason: reason.into(),
        }
    }
}

impl From<FailedContextConversion> for TransitionError {
    /// Converts a [`FailedContextConversion`] into a [`TransitionError::FailedContextConversion`] variant.
    fn from(error: FailedContextConversion) -> Self {
        Self::FailedContextConversion(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_create_error_with_state_names_and_reason_when_new_is_called() {
        let expected_result = FailedContextConversion {
            source_state_name: "StateA".to_string(),
            target_state_name: "StateB".to_string(),
            reason: "Sample reason".to_string(),
        };

        let result = FailedContextConversion::new("StateA", "StateB", "Sample reason");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_transition_error_when_into_is_called() {
        let error = FailedContextConversion::new("StateA", "StateB", "Sample reason");

        let expected_result = TransitionError::FailedContextConversion(error.clone());

        let result: TransitionError = error.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_both_states_and_the_reason_when_displayed() {
        let error = FailedContextConversion::new("StateA", "StateB", "Sample reason");

        let expected_result = "[FailedContextConversion] Failure during transition from 'StateA' to 'StateB', Reason: 'Sample reason'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
