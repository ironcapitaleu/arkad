//! # Request ID
//!
//! Provides the [`RequestId`] of a request and the [`assign_request_id`] middleware.

use std::fmt;

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

/// The response header that carries the [`RequestId`].
pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// The ID of one request, for example `req_019a5f3c7e2b7d4e8f1a2b3c4d5e6f70`.
///
/// The text is `req_` and a UUID v7 in hex, so IDs sort by the time of the request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestId(String);

impl RequestId {
    /// Creates a new [`RequestId`] from the current time and random bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use api::middleware::request_id::RequestId;
    ///
    /// let request_id = RequestId::generate();
    ///
    /// let expected_result = Some("req_");
    ///
    /// let result = request_id.as_str().get(..4);
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn generate() -> Self {
        Self(format!("req_{}", Uuid::now_v7().simple()))
    }

    /// Returns the ID as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn header_value(&self) -> HeaderValue {
        HeaderValue::from_str(&self.0).expect(
            "Given an ID of ASCII letters, digits, and an underscore, building a header value should always succeed",
        )
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Gives the request a new [`RequestId`] and returns it in the `X-Request-Id` header.
///
/// The middleware puts the ID into the request extensions, where handlers and inner middleware
/// read it. It ignores an `X-Request-Id` header that the client sends.
pub async fn assign_request_id(mut request: Request, next: Next) -> Response {
    let request_id = RequestId::generate();
    request.extensions_mut().insert(request_id.clone());
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(REQUEST_ID_HEADER, request_id.header_value());
    response
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_request_id() {
        implements_auto_traits::<RequestId>();
    }

    #[test]
    fn should_hold_36_characters_for_generated_request_id() {
        let request_id = RequestId::generate();

        let expected_result = 36;

        let result = request_id.as_str().len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_differ_between_two_generated_request_ids() {
        let first = RequestId::generate();
        let second = RequestId::generate();

        let result = first;

        assert_ne!(result, second);
    }
}
