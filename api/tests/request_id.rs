use api::config::InternalDocs;
use axum::body::Body;
use axum::http::Request;
use axum::response::Response;
use pretty_assertions::assert_eq;
use tower::ServiceExt;

const CLIENT_REQUEST_ID: &str = "client-chosen-id";

async fn get_health(request: Request<Body>) -> Response {
    api::router(InternalDocs::Disabled)
        .oneshot(request)
        .await
        .expect("Given the router's infallible service, calling it should always succeed")
}

fn request_id_prefix(response: &Response) -> Option<&str> {
    response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|text| text.get(..4))
}

#[tokio::test]
async fn should_answer_a_request_id_that_starts_with_req_for_health_request() {
    let request = Request::get("/health")
        .body(Body::empty())
        .expect("Given a hardcoded method and path, building the request should always succeed");

    let expected_result = Some("req_");

    let response = get_health(request).await;
    let result = request_id_prefix(&response);

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_ignore_the_request_id_of_the_client_when_the_request_carries_x_request_id() {
    let request = Request::get("/health")
        .header("x-request-id", CLIENT_REQUEST_ID)
        .body(Body::empty())
        .expect("Given a hardcoded method, path, and header, building the request should always succeed");

    let expected_result = Some("req_");

    let response = get_health(request).await;
    let result = request_id_prefix(&response);

    assert_eq!(result, expected_result);
}
