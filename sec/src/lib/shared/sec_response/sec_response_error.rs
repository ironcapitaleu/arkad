//! # SEC Response Error Types
//!
//! This module defines error types and reasons for SEC response processing failures.
//! It is used throughout the [`crate::shared::sec_response`] module and by state machine implementations
//! that require robust error reporting for response processing failures.
//!
//! ## Types
//! - [`SecResponseError`]: Error struct containing the [`SecResponseErrorReason`] that caused the failure. This allows precise diagnostics about why a response couldn't be processed.
//! - [`SecResponseErrorReason`]: Enum describing specific reasons for response processing failure, with contextual information embedded in the variants (such as which header contained invalid UTF-8 or network error details).
//!
//! ## Usage
//! These error types are returned by SEC response processing routines and are used in state data modules
//! to provide detailed diagnostics and error handling for HTTP response processing.
//! They are also used as domain errors for the general state machine error logic in [`crate::error`] and may be wrapped by state-level errors.
//!
//! ## See Also
//! - [`crate::shared::sec_response`]: Main SEC response utilities module.
//! - [`crate::error`]: Error types that may reference SEC response errors for reporting.

use thiserror::Error;

/// Error details for SEC response processing failures.
///
/// This struct provides the reason for the failure with embedded contextual information.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[SecResponseError] Response processing failed: {reason}")]
pub struct SecResponseError {
    /// The reason why the response couldn't be processed.
    pub reason: SecResponseErrorReason,
}

impl SecResponseError {
    /// Creates a new `SecResponseError`.
    #[must_use]
    pub const fn new(reason: SecResponseErrorReason) -> Self {
        Self { reason }
    }
}

/// Enum representing the reason for a response processing failure.
///
/// This enum is marked as non-exhaustive to allow for future extension.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecResponseErrorReason {
    /// Invalid UTF-8 encountered in response headers.
    InvalidUtf8InHeader(String),
    /// Network-related error from reqwest.
    NetworkError(String),
    /// Other unspecified error.
    Other(String),
}

impl std::fmt::Display for SecResponseErrorReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUtf8InHeader(header) => write!(
                f,
                "Response header '{header}' contains invalid UTF-8 data and cannot be processed."
            ),
            Self::NetworkError(message) => write!(f, "Network error occurred while processing response: {message}"),
            Self::Other(message) => write!(f, "An unspecified error occurred during response processing: {message}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_format_display_as_expected_when_reason_is_present() {
        let header_name = "content-type";
        let reason = SecResponseErrorReason::InvalidUtf8InHeader(header_name.to_string());
        let response_error = SecResponseError::new(reason.clone());

        let expected_result = format!(
            "[SecResponseError] Response processing failed: {reason}"
        );

        let result = format!("{response_error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_new_error_when_new_is_called() {
        let message = "Test message";
        let reason = SecResponseErrorReason::Other(message.to_string());

        let expected_result = SecResponseError {
            reason: reason.clone(),
        };

        let result = SecResponseError::new(reason);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_reason_correctly_when_invalid_utf8_in_header() {
        let header_name = "content-type";
        let reason = SecResponseErrorReason::InvalidUtf8InHeader(header_name.to_string());

        let expected_result = "Response header 'content-type' contains invalid UTF-8 data and cannot be processed.";

        let result = format!("{reason}");

        assert_eq!(result, expected_result);
    }
}