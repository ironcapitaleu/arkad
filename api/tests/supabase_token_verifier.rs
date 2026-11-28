//! Integration tests of `SupabaseTokenVerifier`.
//!
//! The test against the local Supabase stack needs `supabase start`, `SUPABASE_URL`, and
//! `SUPABASE_SECRET_KEY`. CI starts no Supabase, so the test is `#[ignore]`. Run it with
//! `cargo test -p api --test supabase_token_verifier -- --ignored`.

use api::implementations::SupabaseTokenVerifier;
use api::traits::token_verifier::{TokenError, TokenVerifier};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use storage::AccountId;
use tokio::net::TcpListener;

const TEST_EMAIL: &str = "verifier-test@example.com";

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Set {name} to run the local stack test"))
}

async fn post(client: &reqwest::Client, url: String, secret_key: &str, body: Value) -> Value {
    client
        .post(url)
        .header("apikey", secret_key)
        .json(&body)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .expect("Given a running local stack and its secret key, the Auth call should succeed")
        .json()
        .await
        .expect("Given an Auth response, its body should be JSON")
}

/// Signs in the test user with a code from the admin API, and returns its ID and login token.
async fn sign_in(
    client: &reqwest::Client,
    supabase_url: &str,
    secret_key: &str,
) -> (String, String) {
    let link = post(
        client,
        format!("{supabase_url}/auth/v1/admin/generate_link"),
        secret_key,
        json!({ "type": "magiclink", "email": TEST_EMAIL }),
    )
    .await;
    let session = post(
        client,
        format!("{supabase_url}/auth/v1/verify"),
        secret_key,
        json!({ "type": "email", "email": TEST_EMAIL, "token": link["email_otp"] }),
    )
    .await;
    let field = |pointer: &str| {
        session
            .pointer(pointer)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .expect("Given a successful verify call, the session should always hold the field")
    };
    (field("/user/id"), field("/access_token"))
}

#[tokio::test]
#[ignore = "needs the local Supabase stack: `supabase start`, with an ES256 key in `supabase/signing_keys.json`"]
async fn should_return_the_account_of_the_user_when_the_local_stack_signed_the_token() {
    let supabase_url = env("SUPABASE_URL");
    let client = reqwest::Client::new();
    let (user_id, token) = sign_in(&client, &supabase_url, &env("SUPABASE_SECRET_KEY")).await;
    let verifier = SupabaseTokenVerifier::fetch(&client, &supabase_url)
        .await
        .expect("Given a running local stack, fetching its public keys should always succeed");

    let expected_result = Ok(AccountId::parse(&user_id)
        .expect("Given the user ID that Supabase Auth returned, it should always be a UUID"));

    let result = verifier
        .verify(&token)
        .await
        .map(|verified| verified.account_id);

    assert_eq!(result, expected_result);
}

/// Serves `app` on a free loopback port and returns the base URL.
async fn serve(app: Router) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Given a free port on the loopback interface, binding should always succeed");
    let port = listener
        .local_addr()
        .expect("Given a bound listener, reading its address should always succeed")
        .port();
    tokio::spawn(async move { axum::serve(listener, app).await });
    format!("http://127.0.0.1:{port}")
}

fn jwks_route(body: Value) -> Router {
    Router::new().route(
        "/auth/v1/.well-known/jwks.json",
        get(move || async move { Json(body) }),
    )
}

#[tokio::test]
async fn should_return_failed_key_fetch_when_the_key_endpoint_serves_no_key_set() {
    let supabase_url = serve(jwks_route(json!({ "keys": "x" }))).await;

    // The reason holds the text of `reqwest`, which a patch release can change.
    let expected_result = Err(true);

    let result = SupabaseTokenVerifier::fetch(&reqwest::Client::new(), &supabase_url)
        .await
        .map(|_| ())
        .map_err(|error| matches!(error, TokenError::FailedKeyFetch { .. }));

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_return_failed_key_fetch_when_the_key_endpoint_answers_an_error() {
    let supabase_url = serve(Router::new().route(
        "/auth/v1/.well-known/jwks.json",
        get(|| async { StatusCode::SERVICE_UNAVAILABLE }),
    ))
    .await;

    // The reason holds the text of `reqwest`, which a patch release can change.
    let expected_result = Err(true);

    let result = SupabaseTokenVerifier::fetch(&reqwest::Client::new(), &supabase_url)
        .await
        .map(|_| ())
        .map_err(|error| matches!(error, TokenError::FailedKeyFetch { .. }));

    assert_eq!(result, expected_result);
}
