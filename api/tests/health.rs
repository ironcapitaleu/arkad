use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::response::Response;
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn get_health() -> Response {
    let request = Request::get("/health")
        .body(Body::empty())
        .expect("Given a hardcoded method and path, building the request should always succeed");
    api::router()
        .oneshot(request)
        .await
        .expect("Given the router's infallible service, calling it should always succeed")
}

#[tokio::test]
async fn should_answer_200_for_health_request() {
    let expected_result = StatusCode::OK;

    let result = get_health().await.status();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_json_content_type_for_health_request() {
    let expected_result = Some("application/json");

    let response = get_health().await;
    let result = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok());

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_status_ok_body_for_health_request() {
    let expected_result = json!({ "status": "ok" });

    let body = get_health()
        .await
        .into_body()
        .collect()
        .await
        .expect("Given a fully buffered JSON body, collecting it should always succeed")
        .to_bytes();
    let result: Value = serde_json::from_slice(&body)
        .expect("Given the JSON body of the health handler, parsing it should always succeed");

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_a_request_id_that_starts_with_req_for_health_request() {
    let expected_result = Some("req_");

    let response = get_health().await;
    let result = response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|text| text.get(..4));

    assert_eq!(result, expected_result);
}
