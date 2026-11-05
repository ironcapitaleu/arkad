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

    /// Returns `true` if the request or the read of its body timed out, also for the file that an
    /// [`ErrorReason::FailedFilingDocument`] names.
    #[must_use]
    pub fn is_timed_out(&self) -> bool {
        self.reason.is_timed_out()
    }

    /// Returns `true` if a new attempt of the same request can succeed, also for the file that an
    /// [`ErrorReason::FailedFilingDocument`] names.
    ///
    /// The transient failures are a 503, a 429, a failed or timed-out request, and a failed or
    /// timed-out read of the body. Every other status and an invalid body are permanent.
    #[must_use]
    pub fn is_transient(&self) -> bool {
        self.reason.is_transient()
    }
}

/// Enum representing the reason why an SEC request failed.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorReason {
    /// The request failed at the transport level, for example on a reset connection.
    FailedRequestExecution {
        /// A human-readable description of the transport failure.
        details: String,
    },
    /// The request did not connect, or the response did not start, within the timeout of the
    /// client.
    TimedOutRequest {
        /// A human-readable description of the request that timed out.
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
            Self::FailedRequestExecution { .. } | Self::TimedOutRequest { .. } => false,
        }
    }

    /// Returns `true` if the request or the read of its body timed out, also under a
    /// [`FailedFilingDocument`](Self::FailedFilingDocument).
    fn is_timed_out(&self) -> bool {
        match self {
            Self::TimedOutRequest { .. } => true,
            Self::InvalidResponse { source } => {
                matches!(source.reason, ResponseErrorReason::TimedOutBodyRead { .. })
            }
            Self::FailedFilingDocument { source, .. } => source.is_timed_out(),
            Self::FailedRequestExecution { .. } => false,
        }
    }

    /// Returns `true` if a new attempt of the same request can succeed, also under a
    /// [`FailedFilingDocument`](Self::FailedFilingDocument).
    fn is_transient(&self) -> bool {
        match self {
            Self::FailedRequestExecution { .. } | Self::TimedOutRequest { .. } => true,
            Self::InvalidResponse { source } => matches!(
                source.reason,
                ResponseErrorReason::InvalidStatusCode {
                    status_code: StatusCode::ServiceUnavailable | StatusCode::TooManyRequests
                } | ResponseErrorReason::FailedBodyRead { .. }
                    | ResponseErrorReason::TimedOutBodyRead { .. }
            ),
            Self::FailedFilingDocument { source, .. } => source.is_transient(),
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
            Self::TimedOutRequest { details } => {
                write!(
                    f,
                    "[TimedOutRequest] HTTP request timed out, Reason: '{details}'"
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
        let details = e.to_string();
        let reason = if e.is_timeout() {
            ErrorReason::TimedOutRequest { details }
        } else {
            ErrorReason::FailedRequestExecution { details }
        };
        Self::new(reason)
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
    use crate::tests::fixtures::sample_server::{impatient_client, stalled_server};

    #[tokio::test]
    async fn should_return_a_timed_out_request_when_the_request_times_out() {
        let address = stalled_server(b"").await;
        let client = impatient_client();
        let error = client
            .get(format!("http://{address}/"))
            .send()
            .await
            .expect_err("The server never answers, so the read timeout always ends the request");

        let expected_result = true;

        let reason = FailedSecRequest::from(error).reason;
        let result = matches!(reason, ErrorReason::TimedOutRequest { .. });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_a_failed_request_execution_when_the_request_fails_without_a_timeout() {
        let error = reqwest::Client::new()
            .get("not a url")
            .build()
            .expect_err("A string with no scheme is never a valid URL");

        let expected_result = true;

        let reason = FailedSecRequest::from(error).reason;
        let result = matches!(reason, ErrorReason::FailedRequestExecution { .. });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_reason_is_timed_out_request() {
        let reason = ErrorReason::TimedOutRequest {
            details: "error sending request for url (https://www.sec.gov/)".to_string(),
        };

        let expected_result = "[TimedOutRequest] HTTP request timed out, Reason: 'error sending request for url (https://www.sec.gov/)'";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_timed_out_when_the_request_timed_out() {
        let error = FailedSecRequest::new(ErrorReason::TimedOutRequest {
            details: "error sending request".to_string(),
        });

        let expected_result = true;

        let result = error.is_timed_out();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_timed_out_when_the_body_read_timed_out() {
        let error = FailedSecRequest::from(InvalidSecResponse::new(
            SecResponseErrorReason::TimedOutBodyRead {
                details: "error decoding response body".to_string(),
            },
        ));

        let expected_result = true;

        let result = error.is_timed_out();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_timed_out_when_a_filing_document_timed_out() {
        let error = FailedSecRequest::new(ErrorReason::TimedOutRequest {
            details: "error sending request".to_string(),
        })
        .for_file("alco-20260908_htm.xml");

        let expected_result = true;

        let result = error.is_timed_out();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_not_timed_out_when_the_sec_answers_an_error_status() {
        let error = invalid_status_code(StatusCode::NotFound);

        let expected_result = false;

        let result = error.is_timed_out();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_not_timed_out_when_the_request_fails_without_a_timeout() {
        let error = FailedSecRequest::new(ErrorReason::FailedRequestExecution {
            details: "connection refused".to_string(),
        });

        let expected_result = false;

        let result = error.is_timed_out();

        assert_eq!(result, expected_result);
    }

    fn invalid_response(reason: SecResponseErrorReason) -> FailedSecRequest {
        FailedSecRequest::from(InvalidSecResponse::new(reason))
    }

    #[test]
    fn should_report_transient_when_the_sec_answers_503() {
        let error = invalid_status_code(StatusCode::ServiceUnavailable);

        let expected_result = true;

        let result = error.is_transient();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_transient_when_the_sec_answers_429() {
        let error = invalid_status_code(StatusCode::TooManyRequests);

        let expected_result = true;

        let result = error.is_transient();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_transient_when_the_request_fails_in_transport() {
        let error = FailedSecRequest::new(ErrorReason::FailedRequestExecution {
            details: "connection reset".to_string(),
        });

        let expected_result = true;

        let result = error.is_transient();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_transient_when_the_request_times_out() {
        let error = FailedSecRequest::new(ErrorReason::TimedOutRequest {
            details: "error sending request".to_string(),
        });

        let expected_result = true;

        let result = error.is_transient();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_transient_when_the_body_read_fails() {
        let error = invalid_response(SecResponseErrorReason::FailedBodyRead {
            details: "connection reset".to_string(),
        });

        let expected_result = true;

        let result = error.is_transient();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_transient_when_the_body_read_times_out() {
        let error = invalid_response(SecResponseErrorReason::TimedOutBodyRead {
            details: "error decoding response body".to_string(),
        });

        let expected_result = true;

        let result = error.is_transient();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_transient_when_a_filing_document_gets_503() {
        let error =
            invalid_status_code(StatusCode::ServiceUnavailable).for_file("alco-20260908_htm.xml");

        let expected_result = true;

        let result = error.is_transient();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_not_transient_when_the_sec_answers_404() {
        let error = invalid_status_code(StatusCode::NotFound);

        let expected_result = false;

        let result = error.is_transient();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_not_transient_when_the_sec_answers_403() {
        let error = invalid_status_code(StatusCode::Forbidden);

        let expected_result = false;

        let result = error.is_transient();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_not_transient_when_the_sec_answers_500() {
        let error = invalid_status_code(StatusCode::InternalServerError);

        let expected_result = false;

        let result = error.is_transient();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_not_transient_when_the_body_does_not_parse() {
        let error = invalid_response(SecResponseErrorReason::InvalidBody {
            details: "expected value at line 1 column 1".to_string(),
        });

        let expected_result = false;

        let result = error.is_transient();

        assert_eq!(result, expected_result);
    }

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
    fn should_not_report_unavailable_when_the_sec_answers_404() {
        let error = invalid_status_code(StatusCode::NotFound);

        let expected_result = false;

        let result = error.is_unavailable();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_report_unavailable_when_the_request_fails_in_transport() {
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
