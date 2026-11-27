//! # Token Error
//!
//! Provides the [`TokenError`] for a login token that fails its check.

use jiff::Timestamp;
use storage::InvalidAccountId;
use thiserror::Error;

/// Separates the reasons why a login token fails its check.
///
/// No variant holds the token, because a token is a credential. A `reason` must not hold it either.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TokenError {
    /// The token is not a JWT: it lacks a part, or a part does not decode.
    #[error("[InvalidTokenFormat] Login token is not a valid JWT, Reason: '{reason}'")]
    InvalidTokenFormat {
        /// Why the token is not a JWT.
        reason: String,
    },

    /// The signature matches none of the known signing keys.
    #[error("[InvalidSignature] Login token signature does not match any known key")]
    InvalidSignature,

    /// The header names a signing key that the verifier does not know.
    #[error("[UnknownSigningKey] Login token names an unknown key '{kid}'")]
    UnknownSigningKey {
        /// The key ID from the `kid` header of the token.
        kid: String,
    },

    /// The token passed its `exp` time.
    #[error("[ExpiredToken] Login token expired at '{expired_at}'")]
    ExpiredToken {
        /// The `exp` time of the token.
        expired_at: Timestamp,
    },

    /// A claim of the token fails its check, for example an `aud` without `authenticated`.
    #[error("[InvalidClaim] Login token claim '{claim}' failed its check, Reason: '{reason}'")]
    InvalidClaim {
        /// The name of the claim, for example `aud`.
        claim: String,
        /// Why the claim fails its check.
        reason: String,
    },

    /// The `sub` claim of the token is not an account ID.
    #[error("[InvalidSubject] Login token carries no valid account, Caused by: {source}")]
    InvalidSubject {
        /// Why the `sub` claim is not an account ID.
        #[source]
        source: InvalidAccountId,
    },

    /// The verifier cannot fetch the signing keys, so it cannot check any token.
    #[error("[FailedKeyFetch] Fetching the signing keys failed, Reason: '{reason}'")]
    FailedKeyFetch {
        /// Why the fetch failed.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;
    use jiff::tz::TimeZone;
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_token_error() {
        implements_auto_traits::<TokenError>();
    }

    #[test]
    fn should_name_the_reason_when_displaying_invalid_token_format() {
        let error = TokenError::InvalidTokenFormat {
            reason: "the token has two parts".to_owned(),
        };

        let expected_result = "[InvalidTokenFormat] Login token is not a valid JWT, Reason: 'the token has two parts'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_key_id_when_displaying_unknown_signing_key() {
        let error = TokenError::UnknownSigningKey {
            kid: "key-2".to_owned(),
        };

        let expected_result = "[UnknownSigningKey] Login token names an unknown key 'key-2'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_expiry_when_displaying_expired_token() {
        let expired_at = TimeZone::UTC
            .to_timestamp(date(2026, 11, 15).at(10, 0, 0, 0))
            .expect("Given a valid hardcoded UTC time, building it should always succeed");
        let error = TokenError::ExpiredToken { expired_at };

        let expected_result = "[ExpiredToken] Login token expired at '2026-11-15T10:00:00Z'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_claim_and_the_reason_when_displaying_invalid_claim() {
        let error = TokenError::InvalidClaim {
            claim: "aud".to_owned(),
            reason: "the audience lacks 'authenticated'".to_owned(),
        };

        let expected_result = "[InvalidClaim] Login token claim 'aud' failed its check, Reason: 'the audience lacks 'authenticated''";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_reason_when_displaying_failed_key_fetch() {
        let error = TokenError::FailedKeyFetch {
            reason: "the JWKS endpoint answered 503".to_owned(),
        };

        let expected_result = "[FailedKeyFetch] Fetching the signing keys failed, Reason: 'the JWKS endpoint answered 503'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_no_key_when_displaying_invalid_signature() {
        let error = TokenError::InvalidSignature;

        let expected_result =
            "[InvalidSignature] Login token signature does not match any known key";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_the_account_id_error_when_displaying_invalid_subject() {
        let error = TokenError::InvalidSubject {
            source: InvalidAccountId::new("service-role"),
        };

        let expected_result = "[InvalidSubject] Login token carries no valid account, Caused by: [InvalidAccountId] Account ID 'service-role' is not a UUID";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
