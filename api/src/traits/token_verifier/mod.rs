//! # Token Verifier
//!
//! Provides [`TokenVerifier`], the port that checks a login token and returns the account it
//! belongs to.
//!
//! ## Modules
//!
//! - [`token_error`]: The [`TokenError`] for a login token that fails its check.

use async_trait::async_trait;
use jiff::Timestamp;
use storage::AccountId;

pub mod token_error;

pub use token_error::TokenError;

/// Checks a Supabase login token and returns the account it belongs to.
///
/// A login token is a JWT that Supabase Auth signs. An implementor checks its signature and its
/// claims. `token` is the value of the `Authorization: Bearer <token>` header, without the
/// `Bearer ` prefix.
#[async_trait]
pub trait TokenVerifier: Send + Sync {
    /// Verifies the signature and the claims of a login token.
    ///
    /// # Errors
    ///
    /// Returns a [`TokenError`] if the token is malformed, expired, signed by an unknown key, or
    /// carries a claim that fails a check. Returns [`TokenError::FailedKeyFetch`] if the verifier
    /// cannot fetch the signing keys.
    async fn verify(&self, token: &str) -> Result<VerifiedToken, TokenError>;
}

/// The facts of a login token that passed its check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedToken {
    /// The account of the token, from its `sub` claim.
    pub account_id: AccountId,
    /// The Supabase session of the token, from its `session_id` claim.
    pub session_id: String,
    /// The time the token expires, from its `exp` claim.
    pub expires_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;
    use jiff::tz::TimeZone;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::tests::fixtures::fake_token_verifier::FakeTokenVerifier;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_verified_token() {
        implements_auto_traits::<VerifiedToken>();
    }

    fn verified_token() -> VerifiedToken {
        VerifiedToken {
            account_id: AccountId::from_uuid(Uuid::from_u128(1)),
            session_id: "session-1".to_owned(),
            expires_at: TimeZone::UTC
                .to_timestamp(date(2026, 11, 15).at(11, 0, 0, 0))
                .expect("Given a valid hardcoded UTC time, building it should always succeed"),
        }
    }

    #[tokio::test]
    async fn should_return_the_verified_token_when_the_verifier_accepts_the_token() {
        let verifier = FakeTokenVerifier::default().accepting("good-token", verified_token());
        let token = "good-token";

        let expected_result = Ok(verified_token());

        let result = verifier.verify(token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_with_invalid_signature_when_the_verifier_does_not_know_the_token() {
        let verifier = FakeTokenVerifier::default().accepting("good-token", verified_token());
        let token = "forged-token";

        let expected_result = Err(TokenError::InvalidSignature);

        let result = verifier.verify(token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_error_when_the_verifier_cannot_fetch_the_keys() {
        let error = TokenError::FailedKeyFetch {
            reason: "the JWKS endpoint answered 503".to_owned(),
        };
        let verifier = FakeTokenVerifier::failing(error.clone());
        let token = "good-token";

        let expected_result = Err(error);

        let result = verifier.verify(token).await;

        assert_eq!(result, expected_result);
    }
}
