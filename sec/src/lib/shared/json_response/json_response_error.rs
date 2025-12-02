//! # JSON Response Error Types
//!
//! This module defines error types and reasons for JSON response validation failures.
//! It is used throughout the [`crate::shared::json_response`] module and by state machine
//! implementations that require robust error reporting for response validation failures.
//!
//! ## Types
//! - [`JsonResponseError`]: Error struct containing the [`JsonResponseErrorReason`] that caused the failure. This allows precise diagnostics about why a response couldn't be validated.
//! - [`JsonResponseErrorReason`]: Enum describing specific reasons for validation failure, with contextual information embedded in the variants (such as the invalid status code or content type).
//!
//! ## Usage
//! These error types are returned by JSON response validation routines and are used in state data modules
//! to provide detailed diagnostics and error handling for HTTP response validation.
//! They are also used as domain errors for the general state machine error logic in [`crate::error`] and may be wrapped by state-level errors.
//!
//! ## See Also
//! - [`crate::shared::json_response`]: Main JSON response utilities module.
//! - [`crate::shared::sec_response`]: Underlying SEC response type.
//! - [`crate::error`]: Error types that may reference JSON response errors for reporting.

use reqwest::StatusCode;
use thiserror::Error;

/// Error details for JSON response validation failures.
///
/// This struct provides the reason for the validation failure with embedded contextual information.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[JsonResponseError] Response validation failed: {reason}")]
pub struct JsonResponseError {
    /// The reason why the response validation failed.
    pub reason: JsonResponseErrorReason,
}

impl JsonResponseError {
    /// Creates a new `JsonResponseError`.
    #[must_use]
    pub const fn new(reason: JsonResponseErrorReason) -> Self {
        Self { reason }
    }
}

/// Enum representing the reason for a response validation failure.
///
/// This enum is marked as non-exhaustive to allow for future extension.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonResponseErrorReason {
    /// Response status code indicates failure (not in 2xx range).
    InvalidStatusCode(StatusCode),
    /// Response body is empty when content is expected.
    EmptyResponseBody,
    /// Invalid or unexpected content type.
    InvalidContentType(String),
    /// Response body contains invalid JSON structure.
    InvalidJsonStructure(String),
    /// Other unspecified validation error.
    Other(String),
}

impl std::fmt::Display for JsonResponseErrorReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidStatusCode(status) => write!(
                f,
                "Response status code {status} indicates failure (expected 2xx)."
            ),
            Self::EmptyResponseBody => {
                write!(f, "Response body is empty when content is expected.")
            }
            Self::InvalidContentType(content_type) => {
                write!(f, "Invalid or unexpected content type: {content_type}")
            }
            Self::InvalidJsonStructure(message) => {
                write!(
                    f,
                    "Response body contains invalid JSON structure: {message}"
                )
            }
            Self::Other(message) => {
                write!(f, "An unspecified validation error occurred: {message}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_format_display_as_expected_when_reason_is_present() {
        let status = StatusCode::BAD_REQUEST;
        let reason = JsonResponseErrorReason::InvalidStatusCode(status);
        let validation_error = JsonResponseError::new(reason.clone());

        let expected_result = format!("[JsonResponseError] Response validation failed: {reason}");

        let result = format!("{validation_error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_new_error_when_new_is_called() {
        let reason = JsonResponseErrorReason::EmptyResponseBody;

        let expected_result = JsonResponseError {
            reason: reason.clone(),
        };

        let result = JsonResponseError::new(reason);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_reason_correctly_when_invalid_status_code() {
        let status = StatusCode::NOT_FOUND;
        let reason = JsonResponseErrorReason::InvalidStatusCode(status);

        let expected_result =
            "Response status code 404 Not Found indicates failure (expected 2xx).";

        let result = format!("{reason}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_reason_correctly_when_empty_response_body() {
        let reason = JsonResponseErrorReason::EmptyResponseBody;

        let expected_result = "Response body is empty when content is expected.";

        let result = format!("{reason}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_reason_correctly_when_invalid_content_type() {
        let content_type = "application/pdf";
        let reason = JsonResponseErrorReason::InvalidContentType(content_type.to_string());

        let expected_result = "Invalid or unexpected content type: application/pdf";

        let result = format!("{reason}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_reason_correctly_when_invalid_json_structure() {
        let error_message = "ERROR: Not allowed!";
        let reason = JsonResponseErrorReason::InvalidJsonStructure(error_message.to_string());

        let expected_result = "Response body contains invalid JSON structure: ERROR: Not allowed!";

        let result = format!("{reason}");

        assert_eq!(result, expected_result);
    }
}
