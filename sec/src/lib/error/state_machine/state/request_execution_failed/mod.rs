//! # Request Execution Failed State Error
//!
//! This module defines the [`RequestExecutionFailed`] error type, which represents SEC request execution errors
//! at the state level within the SEC state machine framework. It wraps domain-level [`SecRequestError`]s with additional
//! state context, enabling precise error reporting and handling in state machine workflows.
//!
//! ## Purpose
//! - Enriches domain SEC request execution errors with state information for robust error propagation.
//! - Supports conversion from domain errors and integration into the [`State`](super::State) error enum.
//!
//! ## Types
//! - [`RequestExecutionFailed`]: Struct representing a SEC request execution error with state context.
//!
//! ## Usage
//! Use [`RequestExecutionFailed`] to wrap [`SecRequestError`]s when a SEC request execution failure occurs within a state. This allows
//! downstream error handlers to access both the state context and the underlying domain error.
//!
//! ## Example
//! ```rust
//! use sec::error::state_machine::state::request_execution_failed::RequestExecutionFailed;
//! use sec::shared::sec_request::sec_request_error::{SecRequestError, SecRequestErrorReason};
//! let sec_request_error = SecRequestError::new(SecRequestErrorReason::NetworkError);
//! let state_error = RequestExecutionFailed::new("ExecuteSecRequest", sec_request_error);
//! ```
use thiserror::Error;

use super::State as StateError;
use crate::shared::sec_request::sec_request_error::SecRequestError;
use crate::traits::error::FromDomainError;

/// Error representing a SEC request execution failure at the state level.
///
/// This error type is used to wrap domain-level [`SecRequestError`]s with additional information about
/// the state in which the error occurred, making it suitable for use in state machine error handling.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[RequestExecutionFailed] Failure in State: '{state_name}'. Error: '{sec_request_error}'")]
pub struct RequestExecutionFailed {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level SEC request error.
    #[source]
    pub sec_request_error: SecRequestError,
}

impl RequestExecutionFailed {
    /// Creates a new `RequestExecutionFailed` error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `sec_request_error` - The underlying SEC request error.
    ///
    /// # Returns
    /// A new [`RequestExecutionFailed`] instance with the provided state and domain error context.
    #[must_use]
    pub fn new(
        state_name: impl Into<String>,
        sec_request_error: SecRequestError,
    ) -> Self {
        Self {
            state_name: state_name.into(),
            sec_request_error,
        }
    }

    /// Returns the name of the state where the error occurred.
    #[must_use]
    pub fn state_name(&self) -> &str {
        &self.state_name
    }

    /// Returns a reference to the underlying SEC request error.
    #[must_use]
    pub const fn sec_request_error(&self) -> &SecRequestError {
        &self.sec_request_error
    }
}

/// Converts a state-level [`RequestExecutionFailed`] error into the [`StateError`] enum variant.
impl From<RequestExecutionFailed> for StateError {
    /// Converts an [`RequestExecutionFailed`] into a [`StateError::RequestExecutionFailed`] variant.
    ///
    /// # Arguments
    /// * `val` - The [`RequestExecutionFailed`] error to convert.
    ///
    /// # Returns
    /// A [`StateError`] containing the provided [`RequestExecutionFailed`] error.
    fn from(domain_error: RequestExecutionFailed) -> Self {
        Self::RequestExecutionFailed(domain_error)
    }
}

/// Implements conversion from a domain-level [`SecRequestError`] to a state-level [`RequestExecutionFailed`] error.
///
/// This allows enriching a [`SecRequestError`] with state context for use in state machine error handling.
impl FromDomainError<SecRequestError> for RequestExecutionFailed {
    type DomainErr = SecRequestError;

    /// Constructs a [`RequestExecutionFailed`] from a domain error and state context.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `err` - The domain-level [`SecRequestError`] to wrap.
    ///
    /// # Returns
    /// A new [`RequestExecutionFailed`] instance with the provided context and domain error.
    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::sec_request::sec_request_error::SecRequestErrorReason;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_request_execution_failed_when_new_is_called() {
        let state_name = "ExecuteSecRequest";
        let sec_request_error = SecRequestError::new(SecRequestErrorReason::NetworkError);
        let expected_state_name = state_name;
        let expected_sec_request_error = sec_request_error.clone();

        let expected_result = RequestExecutionFailed {
            state_name: expected_state_name.to_string(),
            sec_request_error: expected_sec_request_error,
        };

        let result = RequestExecutionFailed::new(state_name, sec_request_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_state_name_when_state_name_method_is_called() {
        let state_name = "ExecuteSecRequest";
        let sec_request_error = SecRequestError::new(SecRequestErrorReason::NetworkError);
        let request_execution_failed = RequestExecutionFailed::new(state_name, sec_request_error);

        let expected_result = state_name;

        let result = request_execution_failed.state_name();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_sec_request_error_when_sec_request_error_method_is_called() {
        let state_name = "ExecuteSecRequest";
        let sec_request_error = SecRequestError::new(SecRequestErrorReason::NetworkError);
        let request_execution_failed = RequestExecutionFailed::new(state_name, sec_request_error.clone());

        let expected_result = &sec_request_error;

        let result = request_execution_failed.sec_request_error();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_from_is_called() {
        let state_name = "ExecuteSecRequest";
        let sec_request_error = SecRequestError::new(SecRequestErrorReason::NetworkError);
        let request_execution_failed = RequestExecutionFailed::new(state_name, sec_request_error);

        let expected_result = StateError::RequestExecutionFailed(request_execution_failed.clone());

        let result: StateError = request_execution_failed.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_from_domain_error_when_from_domain_error_is_called() {
        let state_name = "ExecuteSecRequest";
        let sec_request_error = SecRequestError::new(SecRequestErrorReason::TimeoutError);

        let expected_result = RequestExecutionFailed::new(state_name, sec_request_error.clone());

        let result = RequestExecutionFailed::from_domain_error(state_name, sec_request_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_error_message_when_display_is_called() {
        let state_name = "ExecuteSecRequest";
        let sec_request_error = SecRequestError::new(SecRequestErrorReason::NetworkError);
        let request_execution_failed = RequestExecutionFailed::new(state_name, sec_request_error);

        let expected_result = "[RequestExecutionFailed] Failure in State: 'ExecuteSecRequest'. Error: '[SecRequestError] Request failed: Reason: 'The HTTP request failed due to a network error.'.'";

        let result = request_execution_failed.to_string();

        assert_eq!(result, expected_result);
    }
}