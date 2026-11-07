//! # Problem
//!
//! Provides the [`Problem`] body of an error response, as RFC 9457 defines it.

use axum::Json;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

use super::ApiError;

/// The media type of a [`Problem`] body.
pub const PROBLEM_CONTENT_TYPE: &str = "application/problem+json";

/// The start of the `type` of every [`Problem`]. The error code completes it.
pub const PROBLEM_TYPE_PREFIX: &str = "/errors/";

/// The body of an error response, as RFC 9457 defines it, with the fields `code` and `request_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Problem {
    /// A URI reference that names the kind of error, for example `/errors/not_found`.
    #[serde(rename = "type")]
    #[schema(rename = "type", example = "/errors/not_found")]
    pub problem_type: String,
    /// A short summary for people. It can change, so a program must check `code`.
    #[schema(example = "Not found")]
    pub title: String,
    /// The HTTP status of the response.
    #[schema(example = 404)]
    pub status: u16,
    /// An explanation of the failure for people.
    #[schema(example = "The path does not exist.")]
    pub detail: String,
    /// The stable code that a client program checks.
    #[schema(example = "not_found")]
    pub code: String,
    /// The ID of the request. The `X-Request-Id` header carries the same value.
    #[schema(example = "req_019a5f3c7e2b7d4e8f1a2b3c4d5e6f70")]
    pub request_id: String,
}

impl Problem {
    /// Creates a new [`Problem`] body for `error` in the request with the ID `request_id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use api::error::{ApiError, Problem};
    ///
    /// let error = ApiError::NotFound;
    /// let request_id = "req_019a5f3c7e2b7d4e8f1a2b3c4d5e6f70";
    ///
    /// let expected_result = "/errors/not_found";
    ///
    /// let result = Problem::new(error, request_id).problem_type;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn new(error: ApiError, request_id: &str) -> Self {
        Self {
            problem_type: format!("{PROBLEM_TYPE_PREFIX}{}", error.code()),
            title: error.title().to_owned(),
            status: error.status().as_u16(),
            detail: error.detail().to_owned(),
            code: error.code().to_owned(),
            request_id: request_id.to_owned(),
        }
    }
}

impl IntoResponse for Problem {
    /// Answers with the status of the problem and the body as `application/problem+json`.
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, [(CONTENT_TYPE, PROBLEM_CONTENT_TYPE)], Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_problem() {
        implements_auto_traits::<Problem>();
    }

    #[test]
    fn should_serialize_the_rfc_9457_fields_for_not_found() {
        let problem = Problem::new(ApiError::NotFound, "req_019a5f3c7e2b7d4e8f1a2b3c4d5e6f70");

        let expected_result = json!({
            "type": "/errors/not_found",
            "title": "Not found",
            "status": 404,
            "detail": "The path does not exist.",
            "code": "not_found",
            "request_id": "req_019a5f3c7e2b7d4e8f1a2b3c4d5e6f70",
        });

        let result = serde_json::to_value(problem)
            .expect("Given a struct of strings and a number, serializing it should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_answer_problem_json_content_type_when_problem_becomes_a_response() {
        let problem = Problem::new(ApiError::NotFound, "req_019a5f3c7e2b7d4e8f1a2b3c4d5e6f70");

        let expected_result = Some(&HeaderValue::from_static("application/problem+json"));

        let response = problem.into_response();
        let result = response.headers().get(CONTENT_TYPE);

        assert_eq!(result, expected_result);
    }
}
