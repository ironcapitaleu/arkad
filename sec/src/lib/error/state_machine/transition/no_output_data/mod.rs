//! # Missing Output Data Transition Error
//!
//! This module defines the [`MissingOutputData`] error type, which represents missing output data errors
//! at the transition level within the SEC state machine framework. It captures situations where
//! expected output data from an internal state is not available during state transitions.
//!
//! ## Purpose
//! - Reports missing output data errors with both the failing state and internal state context.
//! - Supports conversion to the [`Transition`](super::Transition) error enum for unified error handling.
//!
//! ## Types
//! - [`MissingOutputData`]: Struct representing a missing output data error with state context.
//!
//! ## Usage
//! Use [`MissingOutputData`] when a state transition fails due to missing output data from an internal state.
//! This allows downstream error handlers to identify both the state where the transition failed and
//! the specific internal state that lacks the required output data.
//!
//! ## Example
//! ```rust
//! use sec::error::state_machine::transition::no_output_data::MissingOutputData;
//!
//! let error = MissingOutputData::new("MainState", "InternalState");
//! ```

use thiserror::Error;

use super::Transition as TransitionError;
/// Error representing missing output data during a state transition.
///
/// This error type is used when a state transition fails because the expected output data
/// from an internal state is not available, providing context about both the failing state
/// and the specific internal state that is missing output data.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[MissingOutputData] Failure in State: `{state_name}` during transition. The output data for `{internal_state_name}` is missing."
)]
pub struct MissingOutputData {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The name of the internal state that is missing output data.
    pub internal_state_name: String,
}

impl MissingOutputData {
    /// Creates a new [`MissingOutputData`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the transition failed.
    /// * `internal_state_name` - The name of the internal state missing output data.
    ///
    /// # Returns
    /// A new [`MissingOutputData`] error instance.
    #[must_use]
    pub fn new(
        state_name: &(impl ToString + ?Sized),
        internal_state_name: &(impl ToString + ?Sized),
    ) -> Self {
        Self {
            state_name: state_name.to_string(),
            internal_state_name: internal_state_name.to_string(),
        }
    }
}

/// Converts a transition-level [`MissingOutputData`] error into the [`TransitionError`] enum variant.
impl From<MissingOutputData> for TransitionError {
    /// Converts a [`MissingOutputData`] into a [`TransitionError::NoOutputData`] variant.
    ///
    /// # Arguments
    /// * `domain_error` - The [`MissingOutputData`] error to convert.
    ///
    /// # Returns
    /// A [`TransitionError`] containing the provided [`MissingOutputData`] error.
    fn from(domain_error: MissingOutputData) -> Self {
        Self::NoOutputData(domain_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_missing_output_data_when_new_is_called() {
        let state_name = "TestState";
        let internal_state_name = "InternalState";

        let expected_result = MissingOutputData {
            state_name: state_name.to_string(),
            internal_state_name: internal_state_name.to_string(),
        };

        let result = MissingOutputData::new(state_name, internal_state_name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_missing_output_data_with_string_refs_when_new_is_called() {
        let state_name = &"MainState".to_string();
        let internal_state_name = &"SubState".to_string();

        let expected_result = MissingOutputData {
            state_name: "MainState".to_string(),
            internal_state_name: "SubState".to_string(),
        };

        let result = MissingOutputData::new(state_name, internal_state_name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_transition_error_when_into_is_called() {
        let missing_output_data = MissingOutputData {
            state_name: "FailingState".to_string(),
            internal_state_name: "MissingDataState".to_string(),
        };

        let expected_result = TransitionError::NoOutputData(missing_output_data.clone());

        let result: TransitionError = missing_output_data.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_proper_error_message_when_formatted() {
        let state_name = "TestState";
        let internal_state_name = "InternalState";
        let error = MissingOutputData::new(state_name, internal_state_name);

        let expected_result = "[MissingOutputData] Failure in State: `TestState` during transition. The output data for `InternalState` is missing.";

        let result = format!("{}", error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_equal_when_same_values() {
        let error1 = MissingOutputData::new("State1", "Internal1");
        let error2 = MissingOutputData::new("State1", "Internal1");

        assert_eq!(error1, error2);
    }

    #[test]
    fn should_not_be_equal_when_different_state_names() {
        let error1 = MissingOutputData::new("State1", "Internal1");
        let error2 = MissingOutputData::new("State2", "Internal1");

        assert_ne!(error1, error2);
    }

    #[test]
    fn should_not_be_equal_when_different_internal_state_names() {
        let error1 = MissingOutputData::new("State1", "Internal1");
        let error2 = MissingOutputData::new("State1", "Internal2");

        assert_ne!(error1, error2);
    }
}
