//! # Error
//!
//! Provides the [`ApiError`] that a client sees and its [`Problem`] body.
//!
//! A handler answers with an [`ApiError`]. The
//! [`render_problem`](crate::middleware::problem::render_problem) middleware writes the body,
//! because the body carries the ID of the request, and only the request holds it.
//!
//! ## Modules
//!
//! - [`problem`]: The RFC 9457 [`Problem`] body of an error response.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::shared::pascal_case::InvalidPascalCase;

pub mod problem;

pub use problem::Problem;

/// Separates the failures that a client sees, with one variant per error code of `/v1`.
#[non_exhaustive]
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum ApiError {
    /// No route matches the path of the request.
    #[error("[NotFound] No route matches the path")]
    NotFound,

    /// The server failed to answer the request. The body carries no internal detail.
    #[error("[InternalError] The server failed to answer the request")]
    InternalError,

    /// The route exists in the contract, but the server has no data to answer it yet.
    #[error("[NotImplemented] The server does not answer the route yet")]
    NotImplemented,
}

impl ApiError {
    /// Returns the HTTP status of the response.
    #[must_use]
    pub const fn status(self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotImplemented => StatusCode::NOT_IMPLEMENTED,
        }
    }

    /// Returns the stable code that a client program checks, for example `not_found`.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::InternalError => "internal_error",
            Self::NotImplemented => "not_implemented",
        }
    }

    /// Returns the short summary for people, for example `Not found`.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::NotFound => "Not found",
            Self::InternalError => "Internal error",
            Self::NotImplemented => "Not implemented",
        }
    }

    /// Returns the explanation of the failure for people.
    #[must_use]
    pub const fn detail(self) -> &'static str {
        match self {
            Self::NotFound => "The path does not exist.",
            Self::InternalError => "The server failed to answer the request.",
            Self::NotImplemented => "The server does not answer this route yet.",
        }
    }
}

impl IntoResponse for ApiError {
    /// Answers with the status of the error and an empty body.
    ///
    /// The response carries the error in its extensions.
    /// [`render_problem`](crate::middleware::problem::render_problem) replaces the response with
    /// the [`Problem`] body.
    fn into_response(self) -> Response {
        let mut response = self.status().into_response();
        response.extensions_mut().insert(self);
        response
    }
}

impl From<InvalidPascalCase> for ApiError {
    /// Converts an [`InvalidPascalCase`] into [`ApiError::InternalError`], because the name comes
    /// from the server and not from the client.
    fn from(_: InvalidPascalCase) -> Self {
        Self::InternalError
    }
}

/// Answers a request that no route matches with [`ApiError::NotFound`].
// axum requires an async handler. This one has nothing to await.
#[allow(clippy::unused_async)]
pub async fn not_found() -> ApiError {
    ApiError::NotFound
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

    #[test]
    fn should_become_internal_error_when_converting_invalid_pascal_case() {
        let error = InvalidPascalCase::new("netIncome");

        let expected_result = ApiError::InternalError;

        let result = ApiError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_answer_501_when_not_implemented_becomes_a_response() {
        let error = ApiError::NotImplemented;

        let expected_result = StatusCode::NOT_IMPLEMENTED;

        let result = error.into_response().status();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_for_not_implemented() {
        let error = ApiError::NotImplemented;

        let expected_result = "[NotImplemented] The server does not answer the route yet";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
