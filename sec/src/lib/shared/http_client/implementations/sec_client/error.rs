//! # SEC Client Errors
//!
//! Provides the [`FailedSecRequest`] returned by
//! [`SecClient::execute_sec_request`](crate::shared::http_client::SecClient::execute_sec_request),
//! and its [`ErrorReason`].

use std::fmt;

use thiserror::Error;

use crate::shared::response::implementations::sec_response::error::{
    ErrorReason as ResponseErrorReason, InvalidSecResponse,
};
use crate::shared::status_code::StatusCode;

/// Error indicating that an SEC request could not be completed successfully.
///
/// Wraps the [`ErrorReason`] distinguishing a transport-level failure from a response that arrived
/// but failed SEC validation.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[FailedSecRequest] SEC request failed, Caused by: {reason}")]
pub struct FailedSecRequest {
    /// The reason why the SEC request failed.
    pub reason: ErrorReason,
}

impl FailedSecRequest {
    /// Creates a new [`FailedSecRequest`] error.
    #[must_use]
    pub const fn new(reason: ErrorReason) -> Self {
        Self { reason }
    }

    /// Wraps the reason of this error in [`ErrorReason::FailedFilingDocument`], which names the
    /// file the request fetched.
    #[must_use]
    pub fn for_file(self, file_name: impl Into<String>) -> Self {
        Self::new(ErrorReason::FailedFilingDocument {
            file_name: file_name.into(),
            source: Box::new(self.reason),
        })
    }

    /// Returns `true` if the SEC answered 403 or 404, also for the file that an
    /// [`ErrorReason::FailedFilingDocument`] names.
    ///
    /// For a file that does not exist, the SEC answers 404, or 403 with an S3 `AccessDenied` body,
    /// as observed on 2026-10-30. A 403 can also mean that the SEC refused the request. For a
    /// filing folder that does not exist, the SEC answers 404 or 503, as observed on 2026-11-01.
    /// This method counts the 404 and leaves the 503 to the caller.
    #[must_use]
    pub fn is_absent_file(&self) -> bool {
        self.reason
            .has_status(|status| matches!(status, StatusCode::Forbidden | StatusCode::NotFound))
    }

    /// Returns `true` if the SEC answered 503, also for the file that an
    /// [`ErrorReason::FailedFilingDocument`] names.
    ///
    /// A 503 can mean a short outage, and the SEC also answers it for a filing folder that does
    /// not exist. So a caller that retries a 503 needs a limit on its attempts.
    #[must_use]
    pub fn is_unavailable(&self) -> bool {
        self.reason
            .has_status(|status| matches!(status, StatusCode::ServiceUnavailable))
    }
}

/// Enum representing the reason why an SEC request failed.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorReason {
    /// The request failed at the transport level (network error, timeout, etc.).
    FailedRequestExecution {
        /// A human-readable description of the transport failure.
        details: String,
    },
    /// The response was received but did not meet SEC validity requirements.
    InvalidResponse {
        /// The underlying response-validation error.
        source: InvalidSecResponse,
    },
    /// The request for one file of a filing failed.
    FailedFilingDocument {
        /// The name of the file in the folder of the filing.
        file_name: String,
        /// The reason the request for the file failed.
        source: Box<Self>,
    },
}

impl ErrorReason {
    /// Returns `true` if the SEC answered a status code for which `is_match` returns `true`, also
    /// under a [`FailedFilingDocument`](Self::FailedFilingDocument).
    fn has_status(&self, is_match: impl Fn(&StatusCode) -> bool) -> bool {
        match self {
            Self::InvalidResponse { source } => matches!(
                &source.reason,
                ResponseErrorReason::InvalidStatusCode { status_code } if is_match(status_code)
            ),
            Self::FailedFilingDocument { source, .. } => source.has_status(is_match),
            Self::FailedRequestExecution { .. } => false,
        }
    }
}

impl fmt::Display for ErrorReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FailedRequestExecution { details } => {
                write!(
                    f,
                    "[FailedRequestExecution] HTTP request execution failed, Reason: '{details}'"
                )
            }
            Self::InvalidResponse { source } => {
                write!(
                    f,
                    "[InvalidResponse] Response validation failed, Caused by: {source}"
                )
            }
            Self::FailedFilingDocument { file_name, source } => {
                write!(
                    f,
                    "[FailedFilingDocument] Request for file '{file_name}' failed, Caused by: {source}"
                )
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

    fn invalid_status_code(status_code: StatusCode) -> FailedSecRequest {
        FailedSecRequest::from(InvalidSecResponse::new(
            SecResponseErrorReason::InvalidStatusCode { status_code },
        ))
    }

    #[test]
    fn should_report_an_absent_file_when_the_sec_answers_403_or_404() {
        let forbidden = invalid_status_code(StatusCode::Forbidden);
        let not_found = invalid_status_code(StatusCode::NotFound);

        let expected_result = (true, true);

        let result = (forbidden.is_absent_file(), not_found.is_absent_file());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_an_absent_file_when_a_filing_document_answers_404() {
        let error = invalid_status_code(StatusCode::NotFound).for_file("alco-20260908_htm.xml");

        let expected_result = true;

        let result = error.is_absent_file();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_absent_file_when_the_sec_answers_500() {
        let error = invalid_status_code(StatusCode::InternalServerError);

        let expected_result = false;

        let result = error.is_absent_file();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_absent_file_when_the_request_fails_in_transport() {
        let error = FailedSecRequest::new(ErrorReason::FailedRequestExecution {
            details: "connection refused".to_string(),
        });

        let expected_result = false;

        let result = error.is_absent_file();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_unavailable_when_the_sec_answers_503() {
        let error = invalid_status_code(StatusCode::ServiceUnavailable);

        let expected_result = true;

        let result = error.is_unavailable();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_unavailable_when_a_filing_document_answers_503() {
        let error =
            invalid_status_code(StatusCode::ServiceUnavailable).for_file("alco-20260908_htm.xml");

        let expected_result = true;

        let result = error.is_unavailable();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_not_unavailable_when_the_sec_answers_404() {
        let error = invalid_status_code(StatusCode::NotFound);

        let expected_result = false;

        let result = error.is_unavailable();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_not_unavailable_when_the_request_fails_in_transport() {
        let error = FailedSecRequest::new(ErrorReason::FailedRequestExecution {
            details: "connection refused".to_string(),
        });

        let expected_result = false;

        let result = error.is_unavailable();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_absent_file_when_the_sec_answers_503() {
        let error = invalid_status_code(StatusCode::ServiceUnavailable);

        let expected_result = false;

        let result = error.is_absent_file();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_reason_is_failed_request_execution() {
        let reason = ErrorReason::FailedRequestExecution {
            details: "connection refused".to_string(),
        };
        let error = FailedSecRequest::new(reason.clone());

        let expected_result = format!("[FailedSecRequest] SEC request failed, Caused by: {reason}");

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

        let expected_result = format!("[FailedSecRequest] SEC request failed, Caused by: {reason}");

        let result = format!("{error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_file_and_chain_the_cause_when_displaying_a_failed_filing_document() {
        let error = invalid_status_code(StatusCode::NotFound).for_file("alco-20260908_htm.xml");

        let expected_result = "[FailedSecRequest] SEC request failed, Caused by: [FailedFilingDocument] Request for file 'alco-20260908_htm.xml' failed, Caused by: [InvalidResponse] Response validation failed, Caused by: [InvalidSecResponse] Invalid SEC Response, Reason: 'Status code '404' is not accepted'";

        let result = error.to_string();

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

    #[test]
    fn should_format_display_as_expected_for_failed_request_execution_reason() {
        let reason = ErrorReason::FailedRequestExecution {
            details: "connection refused".to_string(),
        };

        let expected_result =
            "[FailedRequestExecution] HTTP request execution failed, Reason: 'connection refused'";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_for_invalid_response_reason() {
        let sec_response_error =
            InvalidSecResponse::new(SecResponseErrorReason::InvalidContentType {
                content_type: ContentType::Xml,
            });
        let reason = ErrorReason::InvalidResponse {
            source: sec_response_error,
        };

        let expected_result = "[InvalidResponse] Response validation failed, Caused by: [InvalidSecResponse] Invalid SEC Response, Reason: 'Content type 'application/xml' is not accepted'";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }
}
