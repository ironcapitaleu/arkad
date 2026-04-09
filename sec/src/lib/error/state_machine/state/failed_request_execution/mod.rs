//! # Failed Request Execution State Error
//!
//! This module defines the [`FailedRequestExecution`] error type, which represents SEC request execution errors
//! at the state level within the SEC state machine framework. It wraps domain-level [`FailedSecRequest`] errors with
//! additional state context, enabling precise error reporting and handling in state machine workflows.
//!
//! ## Purpose
//! - Enriches domain SEC request execution errors with state information for robust error propagation.
//! - Supports conversion from domain errors and integration into the [`State`](super::State) error enum.
//!
//! ## Types
//! - [`FailedRequestExecution`]: Struct representing a SEC request execution error with state context.
//!
//! ## Usage
//! Use [`FailedRequestExecution`] to wrap [`FailedSecRequest`] errors when a SEC request execution failure
//! occurs within a state. This allows downstream error handlers to access both the state context and
//! the underlying domain error.

use thiserror::Error;

use super::State as StateError;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::traits::error::FromDomainError;

/// Error representing a SEC request execution failure at the state level.
///
/// This error type is used to wrap domain-level [`FailedSecRequest`] errors with additional information about
/// the state in which the error occurred, making it suitable for use in state machine error handling.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[FailedRequestExecution] Failure in State: '{state_name}'. Error: '{domain_error}'")]
pub struct FailedRequestExecution {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level SEC request error.
    #[source]
    pub domain_error: FailedSecRequest,
}

impl FailedRequestExecution {
    /// Creates a new [`FailedRequestExecution`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `domain_error` - The underlying [`FailedSecRequest`] error.
    ///
    /// # Returns
    /// A new [`FailedRequestExecution`] instance with the provided state and domain error context.
    #[must_use]
    pub fn new(state_name: impl Into<String>, domain_error: FailedSecRequest) -> Self {
        Self {
            state_name: state_name.into(),
            domain_error,
        }
    }

    /// Returns the name of the state where the error occurred.
    #[must_use]
    pub fn state_name(&self) -> &str {
        &self.state_name
    }

    /// Returns a reference to the underlying domain error.
    #[must_use]
    pub const fn domain_error(&self) -> &FailedSecRequest {
        &self.domain_error
    }
}

/// Converts a state-level [`FailedRequestExecution`] error into the [`StateError`] enum variant.
impl From<FailedRequestExecution> for StateError {
    fn from(domain_error: FailedRequestExecution) -> Self {
        Self::FailedRequestExecution(domain_error)
    }
}

/// Implements conversion from a domain-level [`FailedSecRequest`] to a state-level [`FailedRequestExecution`] error.
impl FromDomainError<FailedSecRequest> for FailedRequestExecution {
    type DomainErr = FailedSecRequest;

    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::http_client::implementations::sec_client::error::ErrorReason;

    /// Creates a baseline `FailedSecRequest` for use in tests.
    fn create_baseline_domain_error() -> FailedSecRequest {
        FailedSecRequest::new(ErrorReason::FailedRequestExecution {
            details: "Network unreachable".to_string(),
        })
    }

    #[test]
    fn should_create_failed_request_execution_when_new_is_called() {
        let state_name = "ExecuteSecRequest";
        let domain_error = create_baseline_domain_error();

        let expected_result = FailedRequestExecution {
            state_name: state_name.to_string(),
            domain_error: domain_error.clone(),
        };

        let result = FailedRequestExecution::new(state_name, domain_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_state_name_when_state_name_method_is_called() {
        let state_name = "ExecuteSecRequest";
        let domain_error = create_baseline_domain_error();
        let failed_request_execution = FailedRequestExecution::new(state_name, domain_error);

        let expected_result = state_name;

        let result = failed_request_execution.state_name();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_domain_error_when_domain_error_method_is_called() {
        let state_name = "ExecuteSecRequest";
        let domain_error = create_baseline_domain_error();
        let failed_request_execution =
            FailedRequestExecution::new(state_name, domain_error.clone());

        let expected_result = &domain_error;

        let result = failed_request_execution.domain_error();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_from_is_called() {
        let state_name = "ExecuteSecRequest";
        let domain_error = create_baseline_domain_error();
        let failed_request_execution = FailedRequestExecution::new(state_name, domain_error);

        let expected_result = StateError::FailedRequestExecution(failed_request_execution.clone());

        let result: StateError = failed_request_execution.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_from_domain_error_when_from_domain_error_is_called() {
        let state_name = "ExecuteSecRequest";
        let domain_error = create_baseline_domain_error();

        let expected_result = FailedRequestExecution::new(state_name, domain_error.clone());

        let result = FailedRequestExecution::from_domain_error(state_name, domain_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_error_message_when_display_is_called() {
        let state_name = "ExecuteSecRequest";
        let domain_error = create_baseline_domain_error();
        let failed_request_execution = FailedRequestExecution::new(state_name, domain_error);

        let result = failed_request_execution.to_string();

        assert!(!result.is_empty());
    }
}
