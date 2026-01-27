//! # SEC Request Error Types
//!
//! This module defines error types and reasons for SEC request execution failures.
//! It is used throughout the [`crate::shared::sec_request`] module and by state machine implementations
//! that require robust error reporting for HTTP request execution failures.
//!
//! ## Types
//! - [`SecRequestError`]: Error struct containing the [`SecRequestErrorReason`] that caused the failure. This allows precise diagnostics about why a request couldn't be executed.
//! - [`SecRequestErrorReason`]: Enum describing specific reasons for request failure, with contextual information embedded in the variants (such as network errors, HTTP errors, timeouts, or other issues).
//!
//! ## Usage
//! These error types are returned by SEC request execution routines and are used in state data modules
//! to provide detailed diagnostics and error handling for HTTP request execution.
//! They are also used as domain errors for the general state machine error logic in [`crate::error`] and may be wrapped by state-level errors like [`crate::error::state_machine::state::FailedRequestExecution`].
//!
//! ## See Also
//! - [`crate::shared::sec_request`]: Main SEC request utilities module.
//! - [`crate::shared::sec_response`]: Related SEC response error types.
//! - [`crate::error`]: Error types that may reference SEC request errors for reporting.
//! - [`crate::error::state_machine::state::FailedRequestExecution`]: State-level error that wraps `SecRequestError` for error propagation in state machines.

use reqwest::Error as ReqwestError;
use thiserror::Error;

use crate::shared::sec_response::{SecResponseError, SecResponseErrorReason};

/// Error details for SEC request failures.
///
/// This struct provides the reason for the failure with embedded contextual information.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[SecRequestError] Request failed: Reason: '{reason}'.")]
pub struct SecRequestError {
    /// The reason why the request couldn't be processed.
    pub reason: SecRequestErrorReason,
}

impl SecRequestError {
    /// Creates a new `SecRequestError`.
    #[must_use]
    pub const fn new(reason: SecRequestErrorReason) -> Self {
        Self { reason }
    }
}

/// Enum representing the reason for a request failure.
///
/// This enum is marked as non-exhaustive to allow for future extension.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecRequestErrorReason {
    NetworkError(String),
    HttpError(String),
    Timeout(String),
    Other(String),
}

impl From<ReqwestError> for SecRequestError {
    fn from(e: ReqwestError) -> Self {
        if e.is_timeout() {
            Self::new(SecRequestErrorReason::Timeout(e.to_string()))
        } else if e.is_connect() {
            Self::new(SecRequestErrorReason::NetworkError(e.to_string()))
        } else if e.is_status() {
            e.status().map_or_else(
                || Self::new(SecRequestErrorReason::Other(e.to_string())),
                |status| Self::new(SecRequestErrorReason::HttpError(status.to_string())),
            )
        } else {
            Self::new(SecRequestErrorReason::Other(e.to_string()))
        }
    }
}

impl From<SecResponseError> for SecRequestError {
    fn from(e: SecResponseError) -> Self {
        match e.reason {
            SecResponseErrorReason::InvalidUtf8InHeader(header) => {
                Self::new(SecRequestErrorReason::Other(format!(
                    "Response processing failed: Header '{header}' contains invalid UTF-8 data"
                )))
            }
            SecResponseErrorReason::NetworkError(message) => {
                Self::new(SecRequestErrorReason::NetworkError(message))
            }
            SecResponseErrorReason::Other(message) => {
                Self::new(SecRequestErrorReason::Other(message))
            }
        }
    }
}

impl std::fmt::Display for SecRequestErrorReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(message) => write!(
                f,
                "The HTTP request failed due to a network error: {message}."
            ),
            Self::HttpError(status_code) => write!(
                f,
                "The HTTP request failed due to an HTTP error: {status_code}."
            ),
            Self::Timeout(message) => write!(f, "The HTTP request timed out: {message}."),
            Self::Other(message) => write!(
                f,
                "The HTTP request failed for an unknown reason: {message}."
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_format_display_as_expected_when_reason_is_present() {
        let message = "Connection refused";
        let reason = SecRequestErrorReason::NetworkError(message.to_string());
        let request_error = SecRequestError::new(reason.clone());

        let expected_result = format!("[SecRequestError] Request failed: Reason: '{reason}'.");

        let result = format!("{request_error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_new_error_when_new_is_called() {
        let message = "Test message";
        let reason = SecRequestErrorReason::Other(message.to_string());

        let expected_result = SecRequestError {
            reason: reason.clone(),
        };

        let result = SecRequestError::new(reason);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_reason_correctly_when_network_error() {
        let message = "Connection refused";
        let reason = SecRequestErrorReason::NetworkError(message.to_string());

        let expected_result = "The HTTP request failed due to a network error: Connection refused.";

        let result = format!("{reason}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_reason_correctly_when_http_error() {
        let status_code = "404";
        let reason = SecRequestErrorReason::HttpError(status_code.to_string());

        let expected_result = "The HTTP request failed due to an HTTP error: 404.";

        let result = format!("{reason}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_reason_correctly_when_timeout() {
        let message = "Request took too long";
        let reason = SecRequestErrorReason::Timeout(message.to_string());

        let expected_result = "The HTTP request timed out: Request took too long.";

        let result = format!("{reason}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_reason_correctly_when_other() {
        let message = "Unknown error occurred";
        let reason = SecRequestErrorReason::Other(message.to_string());

        let expected_result =
            "The HTTP request failed for an unknown reason: Unknown error occurred.";

        let result = format!("{reason}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_from_sec_response_error_when_invalid_utf8() {
        let header_name = "content-type";
        let sec_response_error = SecResponseError::new(
            SecResponseErrorReason::InvalidUtf8InHeader(header_name.to_string()),
        );

        let expected_result = SecRequestError::new(SecRequestErrorReason::Other(
            "Response processing failed: Header 'content-type' contains invalid UTF-8 data"
                .to_string(),
        ));

        let result = SecRequestError::from(sec_response_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_from_sec_response_error_when_network_error() {
        let message = "Network unreachable";
        let sec_response_error =
            SecResponseError::new(SecResponseErrorReason::NetworkError(message.to_string()));

        let expected_result =
            SecRequestError::new(SecRequestErrorReason::NetworkError(message.to_string()));

        let result = SecRequestError::from(sec_response_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_from_sec_response_error_when_other() {
        let message = "Something went wrong";
        let sec_response_error =
            SecResponseError::new(SecResponseErrorReason::Other(message.to_string()));

        let expected_result =
            SecRequestError::new(SecRequestErrorReason::Other(message.to_string()));

        let result = SecRequestError::from(sec_response_error);

        assert_eq!(result, expected_result);
    }
}
