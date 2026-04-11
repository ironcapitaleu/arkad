//! # Missing Output Data Transition Error
//!
//! This module defines the [`MissingOutput`] error type, which represents missing output data errors
//! at the transition level within the SEC state machine framework. It captures situations where
//! expected output data from a source state is not available during state transitions.
//!
//! ## Purpose
//! - Reports missing output data errors with both the source and target state context.
//! - Supports conversion to the [`Transition`](super::Transition) error enum for unified error handling.
//!
//! ## Types
//! - [`MissingOutput`]: Struct representing a missing output data error with state context.
//!
//! ## Usage
//! Use [`MissingOutput`] when a state transition fails due to missing output data from a source state.
//! This allows downstream error handlers to identify which source state has not computed its output
//! before attempting to transition to the target state.
//!
//! ## Example
//! ```rust
//! use sec::error::state_machine::transition::missing_output::MissingOutput;
//!
//! let error = MissingOutput::new("ValidateCikFormat", "PrepareSecRequest");
//! ```

use thiserror::Error;

use super::Transition as TransitionError;

/// Error representing missing output data during a state transition.
///
/// This error type is used when a state transition fails because the source state
/// has not computed its output data before transitioning to the target state.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[MissingOutput] Failure during transition from '{source_state_name}', Reason: Output data has not been computed before transitioning to '{target_state_name}'"
)]
pub struct MissingOutput {
    /// The name of the source state that has not computed its output data.
    pub source_state_name: String,
    /// The name of the target state that the transition was attempting to reach.
    pub target_state_name: String,
}

impl MissingOutput {
    /// Creates a new [`MissingOutput`] error.
    ///
    /// # Arguments
    /// * `source_state_name` - The name of the source state missing output data.
    /// * `target_state_name` - The name of the target state the transition was attempting.
    ///
    /// # Returns
    /// A new [`MissingOutput`] error instance.
    #[must_use]
    pub fn new(source_state_name: impl Into<String>, target_state_name: impl Into<String>) -> Self {
        Self {
            source_state_name: source_state_name.into(),
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
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn should_create_missing_output_when_new_is_called() {
        let source_state_name = "ValidateCikFormat";
        let target_state_name = "PrepareSecRequest";

        let expected_result = MissingOutput {
            source_state_name: source_state_name.to_string(),
            target_state_name: target_state_name.to_string(),
        };

        let result = MissingOutput::new(source_state_name, target_state_name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_missing_output_with_string_refs_when_new_is_called() {
        let source_state_name = &"ValidateCikFormat".to_string();
        let target_state_name = &"PrepareSecRequest".to_string();

        let expected_result = MissingOutput {
            source_state_name: "ValidateCikFormat".to_string(),
            target_state_name: "PrepareSecRequest".to_string(),
        };

        let result = MissingOutput::new(source_state_name, target_state_name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_transition_error_when_into_is_called() {
        let missing_output = MissingOutput {
            source_state_name: "ValidateCikFormat".to_string(),
            target_state_name: "PrepareSecRequest".to_string(),
        };

        let expected_result = TransitionError::MissingOutput(missing_output.clone());

        let result: TransitionError = missing_output.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_proper_error_message_when_formatted() {
        let error = MissingOutput::new("ValidateCikFormat", "PrepareSecRequest");

        let expected_result = "[MissingOutput] Failure during transition from 'ValidateCikFormat', Reason: Output data has not been computed before transitioning to 'PrepareSecRequest'";

        let result = format!("{error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_equal_when_same_values() {
        let error1 = MissingOutput::new("StateA", "StateB");
        let error2 = MissingOutput::new("StateA", "StateB");

        assert_eq!(error1, error2);
    }

    #[test]
    fn should_not_be_equal_when_different_source_state_names() {
        let error1 = MissingOutput::new("StateA", "StateB");
        let error2 = MissingOutput::new("StateX", "StateB");

        assert_ne!(error1, error2);
    }

    #[test]
    fn should_not_be_equal_when_different_target_state_names() {
        let error1 = MissingOutput::new("StateA", "StateB");
        let error2 = MissingOutput::new("StateA", "StateX");

        assert_ne!(error1, error2);
    }
}
