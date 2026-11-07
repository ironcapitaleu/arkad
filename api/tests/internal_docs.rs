use api::config::InternalDocs;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use pretty_assertions::assert_eq;
use tower::ServiceExt;

async fn get_internal_docs(internal_docs: InternalDocs) -> StatusCode {
    let request = Request::get("/internal/docs/")
        .body(Body::empty())
        .expect("Given a hardcoded method and path, building the request should always succeed");
    api::router(internal_docs)
        .oneshot(request)
        .await
        .expect("Given the router's infallible service, calling it should always succeed")
        .status()
}

#[tokio::test]
async fn should_answer_404_for_internal_docs_when_internal_docs_are_disabled() {
    let internal_docs = InternalDocs::Disabled;

    let expected_result = StatusCode::NOT_FOUND;

    let result = get_internal_docs(internal_docs).await;

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_200_for_internal_docs_when_internal_docs_are_enabled() {
    let internal_docs = InternalDocs::Enabled;

    let expected_result = StatusCode::OK;

    let result = get_internal_docs(internal_docs).await;

    assert_eq!(result, expected_result);
}
