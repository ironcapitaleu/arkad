//! # Failed Output Conversion Error
//!
//! Provides the [`FailedOutputConversion`] error: a source state's output could not be converted
//! into the destination state's input during a transition.

use thiserror::Error;

use super::Transition as TransitionError;

/// Error representing a failed output conversion during a state transition.
///
/// Records both states involved and the reason, so the failed conversion pinpoints which transition
/// broke and why.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[FailedOutputConversion] Failure during transition from '{source_state_name}' to '{target_state_name}', Reason: '{reason}'"
)]
pub struct FailedOutputConversion {
    /// The name of the source state whose output could not be converted.
    pub source_state_name: String,
    /// The name of the target state whose input could not be produced.
    pub target_state_name: String,
    /// Why the conversion failed.
    pub reason: String,
}

impl FailedOutputConversion {
    /// Creates a new [`FailedOutputConversion`] error.
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

impl From<FailedOutputConversion> for TransitionError {
    /// Converts a [`FailedOutputConversion`] into a [`TransitionError::FailedOutputConversion`] variant.
    fn from(error: FailedOutputConversion) -> Self {
        Self::FailedOutputConversion(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_create_error_with_state_names_and_reason_when_new_is_called() {
        let expected_result = FailedOutputConversion {
            source_state_name: "StateA".to_string(),
            target_state_name: "StateB".to_string(),
            reason: "Sample reason".to_string(),
        };

        let result = FailedOutputConversion::new("StateA", "StateB", "Sample reason");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_transition_error_when_into_is_called() {
        let error = FailedOutputConversion::new("StateA", "StateB", "Sample reason");

        let expected_result = TransitionError::FailedOutputConversion(error.clone());

        let result: TransitionError = error.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_both_states_and_the_reason_when_displayed() {
        let error = FailedOutputConversion::new("StateA", "StateB", "Sample reason");

        let expected_result = "[FailedOutputConversion] Failure during transition from 'StateA' to 'StateB', Reason: 'Sample reason'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
