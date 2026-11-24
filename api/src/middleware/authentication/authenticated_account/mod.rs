//! # Authenticated Account
//!
//! Provides the [`AuthenticatedAccount`] extractor, which gives a handler the account of the
//! login token in the `Authorization: Bearer` header.
//!
//! A handler that takes an [`AuthenticatedAccount`] runs only for a request with a valid token.
//! Every other request gets a `401` problem body, so no handler checks the token itself.

use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use serde_json::json;
use storage::AccountId;

use crate::error::ApiError;
use crate::logging::ApiEvent;
use crate::traits::token_verifier::{TokenError, TokenVerifier};

/// The scheme of the `Authorization` header that carries a login token.
const BEARER_SCHEME: &str = "Bearer";

/// The account of a request whose `Authorization: Bearer` header holds a valid login token.
///
/// The extractor reads the token and checks it with the [`TokenVerifier`] of the router state. The
/// state must give an `Arc<dyn TokenVerifier>` through [`FromRef`].
///
/// | Request | Rejection |
/// | ------- | --------- |
/// | No `Authorization` header with the `Bearer` scheme and a token | [`ApiError::MissingCredentials`] |
/// | The verifier rejects the token | [`ApiError::InvalidCredentials`] |
/// | The verifier cannot fetch its signing keys | [`ApiError::InternalError`] |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthenticatedAccount {
    account_id: AccountId,
}

impl AuthenticatedAccount {
    /// Returns the account of the login token, from its `sub` claim.
    #[must_use]
    pub const fn account_id(&self) -> AccountId {
        self.account_id
    }
}

impl<S> FromRequestParts<S> for AuthenticatedAccount
where
    S: Send + Sync,
    Arc<dyn TokenVerifier>: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = bearer_token(&parts.headers).ok_or(ApiError::MissingCredentials)?;
        let verifier = Arc::<dyn TokenVerifier>::from_ref(state);
        let account_id = verifier
            .verify(token)
            .await
            .inspect_err(|error| log_failed_key_fetch(error, parts.uri.path()))?
            .account_id;
        Ok(Self { account_id })
    }
}

/// Logs a [`TokenError::FailedKeyFetch`] as an `error` event, because the client gets a `500`.
///
/// Every other [`TokenError`] is a fault of the client, so it logs nothing.
fn log_failed_key_fetch(error: &TokenError, path: &str) {
    if matches!(error, TokenError::FailedKeyFetch { .. }) {
        tracing::error!(
            event = %ApiEvent::RequestFailed,
            message = %format!(
                "[FailedTokenCheck] Failed to check the login token, Caused by: {error}"
            ),
            context = %json!({ "path": path }),
        );
    }
}

/// Returns the token of the `Authorization: Bearer <token>` header, if the request has one.
///
/// The scheme matches in any case, as RFC 9110 requires. A header value that is not visible ASCII
/// holds no token.
fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    let token = token.trim();
    (scheme.eq_ignore_ascii_case(BEARER_SCHEME) && !token.is_empty()).then_some(token)
}

#[cfg(test)]
mod tests {
    use axum::http::Request;
    use chrono::{DateTime, Utc};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::tests::fixtures::fake_token_verifier::FakeTokenVerifier;
    use crate::traits::token_verifier::VerifiedToken;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_authenticated_account() {
        implements_auto_traits::<AuthenticatedAccount>();
    }

    fn account_id() -> AccountId {
        AccountId::from_uuid(Uuid::from_u128(1))
    }

    fn verifier() -> Arc<dyn TokenVerifier> {
        let verified = VerifiedToken {
            account_id: account_id(),
            session_id: "session-1".to_owned(),
            expires_at: DateTime::<Utc>::MAX_UTC,
        };
        Arc::new(FakeTokenVerifier::default().accepting("good-token", verified))
    }

    /// Returns the parts of a `GET /` request with the given `Authorization` header.
    fn parts(authorization: Option<&str>) -> Parts {
        let mut request = Request::get("/");
        if let Some(value) = authorization {
            request = request.header(AUTHORIZATION, value);
        }
        request
            .body(())
            .expect("Given a hardcoded method, path, and header, building the request should always succeed")
            .into_parts()
            .0
    }

    #[tokio::test]
    async fn should_return_the_account_of_the_token_when_the_verifier_accepts_the_token() {
        let mut parts = parts(Some("Bearer good-token"));
        let state = verifier();

        let expected_result = Ok(AuthenticatedAccount {
            account_id: account_id(),
        });

        let result = AuthenticatedAccount::from_request_parts(&mut parts, &state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_account_of_the_token_when_the_scheme_is_lowercase() {
        let mut parts = parts(Some("bearer good-token"));
        let state = verifier();

        let expected_result = Ok(AuthenticatedAccount {
            account_id: account_id(),
        });

        let result = AuthenticatedAccount::from_request_parts(&mut parts, &state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_reject_with_missing_credentials_when_the_request_has_no_authorization_header() {
        let mut parts = parts(None);
        let state = verifier();

        let expected_result = Err(ApiError::MissingCredentials);

        let result = AuthenticatedAccount::from_request_parts(&mut parts, &state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_reject_with_missing_credentials_when_the_scheme_is_basic() {
        let mut parts = parts(Some("Basic dXNlcjpwYXNz"));
        let state = verifier();

        let expected_result = Err(ApiError::MissingCredentials);

        let result = AuthenticatedAccount::from_request_parts(&mut parts, &state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_reject_with_missing_credentials_when_the_bearer_header_has_no_token() {
        let mut parts = parts(Some("Bearer "));
        let state = verifier();

        let expected_result = Err(ApiError::MissingCredentials);

        let result = AuthenticatedAccount::from_request_parts(&mut parts, &state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_reject_with_invalid_credentials_when_the_verifier_rejects_the_token() {
        let mut parts = parts(Some("Bearer forged-token"));
        let state = verifier();

        let expected_result = Err(ApiError::InvalidCredentials);

        let result = AuthenticatedAccount::from_request_parts(&mut parts, &state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_reject_with_internal_error_when_the_verifier_cannot_fetch_its_keys() {
        let mut parts = parts(Some("Bearer good-token"));
        let state: Arc<dyn TokenVerifier> =
            Arc::new(FakeTokenVerifier::failing(TokenError::FailedKeyFetch {
                reason: "the JWKS endpoint answered 503".to_owned(),
            }));

        let expected_result = Err(ApiError::InternalError);

        let result = AuthenticatedAccount::from_request_parts(&mut parts, &state).await;

        assert_eq!(result, expected_result);
    }
}
