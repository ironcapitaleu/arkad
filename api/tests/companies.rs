use api::config::InternalDocs;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn get(path: &str) -> Response {
    let request = Request::get(path)
        .body(Body::empty())
        .expect("Given a hardcoded method and path, building the request should always succeed");
    api::router(InternalDocs::Disabled)
        .oneshot(request)
        .await
        .expect("Given the router's infallible service, calling it should always succeed")
}

async fn status_and_code(response: Response) -> (StatusCode, Value) {
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("Given a fully buffered problem body, collecting it should always succeed")
        .to_bytes();
    let problem: Value = serde_json::from_slice(&body)
        .expect("Given a problem body, parsing it should always succeed");
    (status, problem["code"].clone())
}

#[tokio::test]
async fn should_answer_501_not_implemented_for_companies_request() {
    let path = "/v1/companies?ticker=GOOGL";

    let expected_result = (StatusCode::NOT_IMPLEMENTED, json!("not_implemented"));

    let result = status_and_code(get(path).await).await;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_501_not_implemented_for_company_request() {
    let path = "/v1/companies/cmp_01JB6Q2W5V8X3K9M7N4P2R1T0S";

    let expected_result = (StatusCode::NOT_IMPLEMENTED, json!("not_implemented"));

    let result = status_and_code(get(path).await).await;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_501_not_implemented_for_facts_request() {
    let path = "/v1/companies/cmp_01JB6Q2W5V8X3K9M7N4P2R1T0S/facts?concept=revenue";

    let expected_result = (StatusCode::NOT_IMPLEMENTED, json!("not_implemented"));

    let result = status_and_code(get(path).await).await;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_501_not_implemented_for_filings_request() {
    let path = "/v1/companies/cmp_01JB6Q2W5V8X3K9M7N4P2R1T0S/filings?form=10-K";

    let expected_result = (StatusCode::NOT_IMPLEMENTED, json!("not_implemented"));

    let result = status_and_code(get(path).await).await;

    assert_eq!(result, expected_result);
}
