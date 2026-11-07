//! # Error
//!
//! Provides the [`ApiError`] that a client sees and the middleware that renders it as a
//! [`Problem`] body.
//!
//! A handler answers with an [`ApiError`]. The [`render_problem`] middleware writes the body,
//! because the body carries the [`RequestId`] of the request, and only the request holds it.
//!
//! ## Modules
//!
//! - [`problem`]: The RFC 9457 [`Problem`] body of an error response.

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::middleware::request_id::RequestId;

pub mod problem;

pub use problem::Problem;

/// Separates the failures that a client sees, with one variant per error code of `/v1`.
#[non_exhaustive]
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum ApiError {
    /// No route matches the path of the request.
    #[error("[NotFound] No route matches the path")]
    NotFound,
}

impl ApiError {
    /// Returns the HTTP status of the response.
    #[must_use]
    pub const fn status(self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }

    /// Returns the stable code that a client program checks, for example `not_found`.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
        }
    }

    /// Returns the short summary for people, for example `Not found`.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::NotFound => "Not found",
        }
    }

    /// Returns the explanation of the failure for people.
    #[must_use]
    pub const fn detail(self) -> &'static str {
        match self {
            Self::NotFound => "The path does not exist.",
        }
    }
}

impl IntoResponse for ApiError {
    /// Answers with the status of the error and an empty body.
    ///
    /// The response carries the error in its extensions. [`render_problem`] replaces the response
    /// with the [`Problem`] body.
    fn into_response(self) -> Response {
        let mut response = self.status().into_response();
        response.extensions_mut().insert(self);
        response
    }
}

/// Answers a request that no route matches with [`ApiError::NotFound`].
// axum requires an async handler. This one has nothing to await.
#[allow(clippy::unused_async)]
pub async fn not_found() -> ApiError {
    ApiError::NotFound
}

/// Replaces the response of an [`ApiError`] with its [`Problem`] body.
///
/// The middleware reads the [`RequestId`] from the request extensions. If the request holds no
/// [`RequestId`], the middleware passes the response on unchanged.
pub async fn render_problem(request: Request, next: Next) -> Response {
    let request_id = request.extensions().get::<RequestId>().cloned();
    let response = next.run(request).await;
    match (response.extensions().get::<ApiError>(), request_id) {
        (Some(&error), Some(request_id)) => {
            Problem::new(error, request_id.as_str()).into_response()
        }
        _ => response,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_api_error() {
        implements_auto_traits::<ApiError>();
    }

    #[test]
    fn should_answer_404_when_not_found_becomes_a_response() {
        let error = ApiError::NotFound;

        let expected_result = StatusCode::NOT_FOUND;

        let result = error.into_response().status();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_carry_the_error_in_the_extensions_when_not_found_becomes_a_response() {
        let error = ApiError::NotFound;

        let expected_result = Some(&ApiError::NotFound);

        let response = error.into_response();
        let result = response.extensions().get::<ApiError>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_for_not_found() {
        let error = ApiError::NotFound;

        let expected_result = "[NotFound] No route matches the path";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
