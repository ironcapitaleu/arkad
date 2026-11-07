use api::config::InternalDocs;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::response::Response;
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn get(path: &str, internal_docs: InternalDocs) -> Response {
    let request = Request::get(path)
        .body(Body::empty())
        .expect("Given a hardcoded method and path, building the request should always succeed");
    api::router(internal_docs)
        .oneshot(request)
        .await
        .expect("Given the router's infallible service, calling it should always succeed")
}

fn header_text<'a>(response: &'a Response, name: &str) -> Option<&'a str> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
}

#[tokio::test]
async fn should_answer_404_when_v1_path_is_unknown() {
    let path = "/v1/unknown";

    let expected_result = StatusCode::NOT_FOUND;

    let result = get(path, InternalDocs::Disabled).await.status();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_problem_json_content_type_when_v1_path_is_unknown() {
    let path = "/v1/unknown";

    let expected_result = Some("application/problem+json");

    let response = get(path, InternalDocs::Disabled).await;
    let result = header_text(&response, header::CONTENT_TYPE.as_str());

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_not_found_problem_with_the_request_id_of_the_header_when_v1_path_is_unknown()
{
    let path = "/v1/unknown";
    let response = get(path, InternalDocs::Disabled).await;
    let request_id = header_text(&response, "x-request-id")
        .expect("Given the request ID middleware, every response should always carry X-Request-Id")
        .to_owned();

    let expected_result = json!({
        "type": "/errors/not_found",
        "title": "Not found",
        "status": 404,
        "detail": "The path does not exist.",
        "code": "not_found",
        "request_id": request_id,
    });

    let body = response
        .into_body()
        .collect()
        .await
        .expect("Given a fully buffered JSON body, collecting it should always succeed")
        .to_bytes();
    let result: Value = serde_json::from_slice(&body)
        .expect("Given the problem body of the fallback, parsing it should always succeed");

    assert_eq!(result, expected_result);
}
