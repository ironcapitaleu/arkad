//! # SEC Response Errors
//!
//! Provides the [`InvalidSecResponse`] raised when an HTTP response fails SEC validation, and its
//! [`ErrorReason`].

use std::fmt;

use thiserror::Error;

use crate::shared::content_type::ContentType;
use crate::shared::status_code::StatusCode;

/// Reports that an HTTP response could not be accepted as a valid SEC response.
///
/// Wraps the [`ErrorReason`] identifying which validation check failed.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidSecResponse] Invalid SEC Response, Reason: '{reason}'")]
pub struct InvalidSecResponse {
    /// Why the response is considered invalid.
    pub reason: ErrorReason,
}

impl InvalidSecResponse {
    /// Creates a new error from its reason.
    #[must_use]
    pub const fn new(reason: ErrorReason) -> Self {
        Self { reason }
    }
}

/// Why an HTTP response failed SEC validation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorReason {
    /// The status code is not in the success range (200–299).
    InvalidStatusCode {
        /// The non-success status code received.
        status_code: StatusCode,
    },
    /// The content type is not `application/json`.
    InvalidContentType {
        /// The unexpected content type received.
        content_type: ContentType,
    },
    /// The body could not be parsed as valid JSON.
    InvalidBody {
        /// The parser's description of the failure.
        details: String,
    },
    /// The body could not be read from the HTTP response.
    FailedBodyRead {
        /// A description of the read failure.
        details: String,
    },
}

impl fmt::Display for ErrorReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidStatusCode { status_code } => {
                write!(
                    f,
                    "Expected a success status code (2xx), got '{status_code}' status code instead"
                )
            }
            Self::InvalidContentType { content_type } => {
                write!(
                    f,
                    "Expected content type 'application/json', got '{content_type}' instead"
                )
            }
            Self::InvalidBody { details } => {
                write!(f, "Response body is not valid JSON: '{details}'")
            }
            Self::FailedBodyRead { details } => {
                write!(f, "Failed to read response body: '{details}'")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_format_display_as_expected_when_reason_is_invalid_status_code() {
        let reason = ErrorReason::InvalidStatusCode {
            status_code: StatusCode::NotFound,
        };
        let error = InvalidSecResponse::new(reason.clone());

        let expected_result =
            format!("[InvalidSecResponse] Invalid SEC Response, Reason: '{reason}'");

        let result = format!("{error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_reason_is_invalid_content_type() {
        let reason = ErrorReason::InvalidContentType {
            content_type: ContentType::Xml,
        };
        let error = InvalidSecResponse::new(reason.clone());

        let expected_result =
            format!("[InvalidSecResponse] Invalid SEC Response, Reason: '{reason}'");

        let result = format!("{error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_reason_is_invalid_body() {
        let reason = ErrorReason::InvalidBody {
            details: "expected value at line 1 column 1".to_string(),
        };
        let error = InvalidSecResponse::new(reason.clone());

        let expected_result =
            format!("[InvalidSecResponse] Invalid SEC Response, Reason: '{reason}'");

        let result = format!("{error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_reason_is_failed_body_read() {
        let reason = ErrorReason::FailedBodyRead {
            details: "connection reset".to_string(),
        };
        let error = InvalidSecResponse::new(reason.clone());

        let expected_result =
            format!("[InvalidSecResponse] Invalid SEC Response, Reason: '{reason}'");

        let result = format!("{error}");

        assert_eq!(result, expected_result);
    }
}
