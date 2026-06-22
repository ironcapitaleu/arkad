//! # Failed Request Execution Error
//!
//! Provides the [`FailedRequestExecution`] error: a domain-level [`FailedSecRequest`] enriched with
//! the name of the state in which the SEC request failed.

use thiserror::Error;

use super::State as StateError;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::traits::error::FromDomainError;

/// A SEC request execution failure, tagged with the state it occurred in.
///
/// Wraps a domain-level [`FailedSecRequest`] together with the failing state's name, so a
/// transport-level failure carries state context as it propagates up the error hierarchy.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[FailedRequestExecution] Failure in State: '{state_name}', Caused by: {domain_error}")]
pub struct FailedRequestExecution {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level SEC request error.
    #[source]
    pub domain_error: FailedSecRequest,
}

impl FailedRequestExecution {
    /// Creates a new error from the failing state's name and the underlying request error.
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
