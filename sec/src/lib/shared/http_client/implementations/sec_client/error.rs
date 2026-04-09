use std::fmt;

use thiserror::Error;

use crate::shared::response::implementations::sec_response::error::InvalidSecResponse;

/// Error details for a failed SEC request execution.
///
/// This struct provides the reason why the SEC request could not be
/// executed successfully.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("[FailedSecRequest] SEC request failed: Reason: '{reason}'.")]
pub struct FailedSecRequest {
    /// The reason why the SEC request failed.
    pub reason: ErrorReason,
}

impl FailedSecRequest {
    /// Creates a new `FailedSecRequest`.
    #[must_use]
    pub const fn new(reason: ErrorReason) -> Self {
        Self { reason }
    }
}

/// Enum representing the reason for a failed SEC request.
///
/// This enum is marked as non-exhaustive to allow for future extension.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorReason {
    /// The HTTP request failed at the transport level (network error, timeout, etc.).
    FailedRequestExecution { details: String },
    /// The HTTP response was received but did not meet SEC response validity requirements.
    InvalidResponse { source: InvalidSecResponse },
}

impl fmt::Display for ErrorReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailedRequestExecution { details } => {
                write!(f, "HTTP request execution failed: '{details}'")
            }
            Self::InvalidResponse { source } => {
                write!(f, "Response validation failed: '{source}'")
            }
        }
    }
}

impl From<reqwest::Error> for FailedSecRequest {
    fn from(e: reqwest::Error) -> Self {
        Self::new(ErrorReason::FailedRequestExecution {
            details: e.to_string(),
        })
    }
}

impl From<InvalidSecResponse> for FailedSecRequest {
    fn from(e: InvalidSecResponse) -> Self {
        Self::new(ErrorReason::InvalidResponse { source: e })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::shared::content_type::ContentType;
    use crate::shared::response::implementations::sec_response::error::{
        ErrorReason as SecResponseErrorReason, InvalidSecResponse,
    };

    use super::*;

    #[test]
    fn should_format_display_as_expected_when_reason_is_failed_request_execution() {
        let reason = ErrorReason::FailedRequestExecution {
            details: "connection refused".to_string(),
        };
        let error = FailedSecRequest::new(reason.clone());

        let expected_result = format!("[FailedSecRequest] SEC request failed: Reason: '{reason}'.");

        let result = format!("{error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_reason_is_invalid_response() {
        let sec_response_error =
            InvalidSecResponse::new(SecResponseErrorReason::InvalidContentType {
                content_type: ContentType::Xml,
            });
        let reason = ErrorReason::InvalidResponse {
            source: sec_response_error,
        };
        let error = FailedSecRequest::new(reason.clone());

        let expected_result = format!("[FailedSecRequest] SEC request failed: Reason: '{reason}'.");

        let result = format!("{error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_from_invalid_sec_response_when_using_from() {
        let sec_response_error =
            InvalidSecResponse::new(SecResponseErrorReason::InvalidContentType {
                content_type: ContentType::Html,
            });

        let expected_result = ErrorReason::InvalidResponse {
            source: sec_response_error.clone(),
        };

        let result = FailedSecRequest::from(sec_response_error).reason;

        assert_eq!(result, expected_result);
    }
}
