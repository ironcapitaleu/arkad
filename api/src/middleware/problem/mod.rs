//! # Problem
//!
//! Provides the [`render_problem`] middleware, which writes the [`Problem`] body of an
//! [`ApiError`].

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::error::{ApiError, Problem};
use crate::middleware::request_id::RequestId;

/// Replaces the response of an [`ApiError`] with its [`Problem`] body.
///
/// The body carries the [`RequestId`] from the request extensions. If the request holds no
/// [`RequestId`], the middleware creates one, so an error response always has a body. That ID
/// matches no `X-Request-Id` header.
pub async fn render_problem(request: Request, next: Next) -> Response {
    let request_id = request.extensions().get::<RequestId>().cloned();
    let response = next.run(request).await;
    let Some(&error) = response.extensions().get::<ApiError>() else {
        return response;
    };
    let request_id = request_id.unwrap_or_else(RequestId::generate);
    Problem::new(error, request_id.as_str()).into_response()
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::middleware::from_fn;
    use http_body_util::BodyExt;
    use pretty_assertions::assert_eq;
    use serde_json::Value;
    use tower::ServiceExt;

    use super::*;
    use crate::error::not_found;

    #[tokio::test]
    async fn should_write_the_problem_body_when_the_request_holds_no_request_id() {
        let router = Router::new()
            .fallback(not_found)
            .layer(from_fn(render_problem));
        let request = Request::get("/unknown").body(Body::empty()).expect(
            "Given a hardcoded method and path, building the request should always succeed",
        );

        let expected_result = Some("not_found");

        let response = router
            .oneshot(request)
            .await
            .expect("Given the router's infallible service, calling it should always succeed");
        let body = response
            .into_body()
            .collect()
            .await
            .expect("Given a fully buffered JSON body, collecting it should always succeed")
            .to_bytes();
        let problem: Value = serde_json::from_slice(&body)
            .expect("Given a problem body, parsing it should always succeed");
        let result = problem["code"].as_str();

        assert_eq!(result, expected_result);
    }
}
