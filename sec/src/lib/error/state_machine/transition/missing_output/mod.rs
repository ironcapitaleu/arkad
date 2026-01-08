//! # Missing Output Data Transition Error
//!
//! This module defines the [`MissingOutput`] error type, which represents missing output data errors
//! at the transition level within the SEC state machine framework. It captures situations where
//! expected output data from a source state is not available during state transitions.
//!
//! ## Purpose
//! - Reports missing output data errors with both the failing state and source state context.
//! - Supports conversion to the [`Transition`](super::Transition) error enum for unified error handling.
//!
//! ## Types
//! - [`MissingOutput`]: Struct representing a missing output data error with state context.
//!
//! ## Usage
//! Use [`MissingOutput`] when a state transition fails due to missing output data from a source state.
//! This allows downstream error handlers to identify both the state where the transition failed and
//! the specific source state that lacks the required output data.
//!
//! ## Example
//! ```rust
//! use sec::error::state_machine::transition::missing_output::MissingOutput;
//!
//! let error = MissingOutput::new("MainState", "SourceState");
//! ```

use thiserror::Error;

use super::Transition as TransitionError;
/// Error representing missing output data during a state transition.
///
/// This error type is used when a state transition fails because the expected output data
/// from a source state is not available, providing context about both the failing state
/// and the specific source state that is missing output data.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[MissingOutput] Failure in State: `{super_state_name}` during transition. The output data for `{target_state_name}` is missing."
)]
pub struct MissingOutput {
    /// The name of the super state where the error occurred.
    pub super_state_name: String,
    /// The name of the target state that is missing output data.
    pub target_state_name: String,
}

impl MissingOutput {
    /// Creates a new [`MissingOutput`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the transition failed.
    /// * `source_state_name` - The name of the source state missing output data.
    ///
    /// # Returns
    /// A new [`MissingOutput`] error instance.
    #[must_use]
    pub fn new(super_state_name: impl Into<String>, target_state_name: impl Into<String>) -> Self {
        Self {
            super_state_name: super_state_name.into(),
            target_state_name: target_state_name.into(),
        }
    }
}

/// Converts a transition-level [`MissingOutput`] error into the [`TransitionError`] enum variant.
impl From<MissingOutput> for TransitionError {
    /// Converts a [`MissingOutput`] into a [`TransitionError::MissingOutput`] variant.
    ///
    /// # Arguments
    /// * `error` - The [`MissingOutput`] error to convert.
    ///
    /// # Returns
    /// A [`TransitionError`] containing the provided [`MissingOutput`] error.
    fn from(error: MissingOutput) -> Self {
        Self::MissingOutput(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_missing_output_when_new_is_called() {
        let super_state_name = "TestState";
        let target_state_name = "TargetState";

        let expected_result = MissingOutput {
            super_state_name: super_state_name.to_string(),
            target_state_name: target_state_name.to_string(),
        };

        let result = MissingOutput::new(super_state_name, target_state_name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_missing_output_with_string_refs_when_new_is_called() {
        let super_state_name = &"MainState".to_string();
        let target_state_name = &"SubState".to_string();

        let expected_result = MissingOutput {
            super_state_name: "MainState".to_string(),
            target_state_name: "SubState".to_string(),
        };

        let result = MissingOutput::new(super_state_name, target_state_name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_transition_error_when_into_is_called() {
        let missing_output = MissingOutput {
            super_state_name: "FailingState".to_string(),
            target_state_name: "MissingDataState".to_string(),
        };

        let expected_result = TransitionError::MissingOutput(missing_output.clone());

        let result: TransitionError = missing_output.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_proper_error_message_when_formatted() {
        let super_state_name = "TestState";
        let target_state_name = "TargetState";
        let error = MissingOutput::new(super_state_name, target_state_name);

        let expected_result = "[MissingOutput] Failure in State: `TestState` during transition. The output data for `TargetState` is missing.";

        let result = format!("{}", error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_equal_when_same_values() {
        let error1 = MissingOutput::new("State1", "Source1");
        let error2 = MissingOutput::new("State1", "Source1");

        assert_eq!(error1, error2);
    }

    #[test]
    fn should_not_be_equal_when_different_state_names() {
        let error1 = MissingOutput::new("State1", "Source1");
        let error2 = MissingOutput::new("State2", "Source1");

        assert_ne!(error1, error2);
    }

    #[test]
    fn should_not_be_equal_when_different_source_state_names() {
        let error1 = MissingOutput::new("State1", "Source1");
        let error2 = MissingOutput::new("State1", "Source2");

        assert_ne!(error1, error2);
    }
}
