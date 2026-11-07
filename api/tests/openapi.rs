//! # Spec Tests
//!
//! Compares the generated spec with the committed `api/openapi.json`, and checks the spec
//! that `GET /v1/openapi.json` serves.
//!
//! ## Writing the Committed Spec
//!
//! If a change to a route changes the spec, write the file again and commit it:
//!
//! ```sh
//! UPDATE_OPENAPI=1 cargo test -p api --test openapi
//! ```

use std::env;
use std::fs;

use api::config::InternalDocs;
use axum::body::Body;
use axum::http::{Request, header};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use serde_json::Value;
use tower::ServiceExt;

const COMMITTED_SPEC_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/openapi.json");

fn generated_spec() -> String {
    let spec = api::openapi().to_pretty_json().expect(
        "Given a spec of JSON values with string keys, serializing it should always succeed",
    );
    format!("{spec}\n")
}

#[test]
fn should_match_the_committed_spec_for_generated_spec() {
    if env::var_os("UPDATE_OPENAPI").is_some() {
        fs::write(COMMITTED_SPEC_PATH, generated_spec())
            .expect("Given the writable crate folder, writing the spec should always succeed");
    }
    let committed_spec = fs::read_to_string(COMMITTED_SPEC_PATH).expect(
        "Given the committed api/openapi.json, reading it should always succeed. Write it with UPDATE_OPENAPI=1",
    );

    let expected_result = committed_spec.trim_end();

    let result = api::openapi().to_pretty_json().expect(
        "Given a spec of JSON values with string keys, serializing it should always succeed",
    );

    assert_eq!(result, expected_result);
}

#[test]
fn should_declare_openapi_3_1_for_generated_spec() {
    let spec: Value = serde_json::from_str(&generated_spec())
        .expect("Given the JSON text of the spec, parsing it should always succeed");

    let expected_result = Some("3.1.0");

    let result = spec["openapi"].as_str();

    assert_eq!(result, expected_result);
}

#[test]
fn should_list_only_the_v1_paths_for_generated_spec() {
    let spec: Value = serde_json::from_str(&generated_spec())
        .expect("Given the JSON text of the spec, parsing it should always succeed");

    let expected_result = vec!["/v1/concepts", "/v1/openapi.json"];

    let result: Vec<&str> = spec["paths"]
        .as_object()
        .expect("Given a spec, its paths should always be a JSON object")
        .keys()
        .map(String::as_str)
        .collect();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_serve_the_generated_spec_for_openapi_json_request() {
    let request = Request::get("/v1/openapi.json")
        .body(Body::empty())
        .expect("Given a hardcoded method and path, building the request should always succeed");

    let expected_result: Value = serde_json::from_str(&generated_spec())
        .expect("Given the JSON text of the spec, parsing it should always succeed");

    let response = api::router(InternalDocs::Disabled)
        .oneshot(request)
        .await
        .expect("Given the router's infallible service, calling it should always succeed");
    let body = response
        .into_body()
        .collect()
        .await
        .expect("Given a fully buffered body, collecting it should always succeed")
        .to_bytes();
    let result: Value = serde_json::from_slice(&body)
        .expect("Given the body of the spec handler, parsing it should always succeed");

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_json_content_type_for_openapi_json_request() {
    let request = Request::get("/v1/openapi.json")
        .body(Body::empty())
        .expect("Given a hardcoded method and path, building the request should always succeed");

    let expected_result = Some("application/json");

    let response = api::router()
        .oneshot(request)
        .await
        .expect("Given the router's infallible service, calling it should always succeed");
    let result = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok());

    assert_eq!(result, expected_result);
}
