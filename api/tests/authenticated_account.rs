//! # Authenticated Account Integration Tests
//!
//! Sends requests through a router with a route that takes an `AuthenticatedAccount`, the problem
//! middleware, and the request ID middleware. A `FakeTokenVerifier` in the router state checks the
//! tokens.
//!
//! ## External Dependencies
//!
//! - None. The tests run in memory.

use std::sync::Arc;

use api::AppState;
use api::middleware::authentication::AuthenticatedAccount;
use api::middleware::problem::render_problem;
use api::middleware::request_id::assign_request_id;
use api::tests::fixtures::fake_token_verifier::FakeTokenVerifier;
use api::traits::token_verifier::VerifiedToken;
use axum::Router;
use axum::body::Body;
use axum::http::header::AUTHORIZATION;
use axum::http::{Request, StatusCode};
use axum::middleware::from_fn;
use axum::response::Response;
use axum::routing::get;
use chrono::{DateTime, Utc};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use storage::AccountId;
use tower::ServiceExt;
use uuid::Uuid;

const ACCOUNT_PATH: &str = "/account";
const VALID_TOKEN: &str = "valid-token";

/// Answers with the account ID of the caller.
async fn account(account: AuthenticatedAccount) -> String {
    account.account_id().to_string()
}

fn router() -> Router {
    let verified_token = VerifiedToken {
        account_id: AccountId::from_uuid(Uuid::from_u128(1)),
        session_id: "session-1".to_owned(),
        expires_at: DateTime::<Utc>::MAX_UTC,
    };
    let verifier = FakeTokenVerifier::default().accepting(VALID_TOKEN, verified_token);
    Router::new()
        .route(ACCOUNT_PATH, get(account))
        .layer(from_fn(render_problem))
        .layer(from_fn(assign_request_id))
        .with_state(AppState::new(Arc::new(verifier)))
}

async fn get_account(authorization: Option<&str>) -> Response {
    let mut request = Request::get(ACCOUNT_PATH);
    if let Some(value) = authorization {
        request = request.header(AUTHORIZATION, value);
    }
    let request = request.body(Body::empty()).expect(
        "Given a hardcoded method, path, and header, building the request should always succeed",
    );
    router()
        .oneshot(request)
        .await
        .expect("Given the router's infallible service, calling it should always succeed")
}

fn request_id(response: &Response) -> String {
    response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .expect("Given the request ID middleware, every response should always carry X-Request-Id")
        .to_owned()
}

async fn body_bytes(response: Response) -> Vec<u8> {
    response
        .into_body()
        .collect()
        .await
        .expect("Given a fully buffered body, collecting it should always succeed")
        .to_bytes()
        .to_vec()
}

#[tokio::test]
async fn should_give_the_handler_the_account_of_the_token_when_the_token_is_valid() {
    let authorization = Some("Bearer valid-token");

    let expected_result = (
        StatusCode::OK,
        "00000000-0000-0000-0000-000000000001".to_owned(),
    );

    let response = get_account(authorization).await;
    let status = response.status();
    let body = String::from_utf8(body_bytes(response).await)
        .expect("Given the text body of the handler, it should always be UTF-8");
    let result = (status, body);

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_401_missing_credentials_problem_when_the_request_has_no_authorization_header()
 {
    let authorization = None;
    let response = get_account(authorization).await;
    let request_id = request_id(&response);

    let expected_result = (
        StatusCode::UNAUTHORIZED,
        json!({
            "type": "/errors/missing_credentials",
            "title": "Missing credentials",
            "status": 401,
            "detail": "The request has no `Authorization: Bearer` header.",
            "code": "missing_credentials",
            "request_id": request_id,
        }),
    );

    let status = response.status();
    let body: Value = serde_json::from_slice(&body_bytes(response).await)
        .expect("Given the problem body of the extractor, parsing it should always succeed");
    let result = (status, body);

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_401_invalid_credentials_problem_when_the_verifier_rejects_the_token() {
    let authorization = Some("Bearer forged-token");
    let response = get_account(authorization).await;
    let request_id = request_id(&response);

    let expected_result = (
        StatusCode::UNAUTHORIZED,
        json!({
            "type": "/errors/invalid_credentials",
            "title": "Invalid credentials",
            "status": 401,
            "detail": "The API key is unknown, or the login token is malformed, expired, or not signed by Supabase.",
            "code": "invalid_credentials",
            "request_id": request_id,
        }),
    );

    let status = response.status();
    let body: Value = serde_json::from_slice(&body_bytes(response).await)
        .expect("Given the problem body of the extractor, parsing it should always succeed");
    let result = (status, body);

    assert_eq!(result, expected_result);
}
