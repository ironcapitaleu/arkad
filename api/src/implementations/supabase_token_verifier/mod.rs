//! # Supabase Token Verifier
//!
//! Provides [`SupabaseTokenVerifier`], the [`TokenVerifier`] that checks Supabase login tokens
//! against the public keys of a Supabase project.
//!
//! ## Modules
//!
//! - [`supabase_claims`]: The [`SupabaseClaims`] of a login token and their checks.

use std::collections::HashMap;
use std::fmt::Display;

use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::errors::{Error as JwtError, ErrorKind};
use jsonwebtoken::jwk::{
    AlgorithmParameters, EllipticCurve, Jwk, JwkSet, KeyAlgorithm, PublicKeyUse,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};

use crate::traits::token_verifier::{TokenError, TokenVerifier, VerifiedToken};

pub mod supabase_claims;

pub use supabase_claims::SupabaseClaims;

/// The path of the public keys below the URL of a Supabase project.
const JWKS_PATH: &str = "/auth/v1/.well-known/jwks.json";

/// The path of Supabase Auth below the URL of a Supabase project, which is the token issuer.
const AUTH_PATH: &str = "/auth/v1";

/// A [`TokenVerifier`] that checks a Supabase login token with the public keys of the project.
///
/// It accepts tokens that an `ES256` or an `RS256` key signs, and rejects every other algorithm,
/// such as `HS256` and `none`. It checks the signature before it reads any claim. The claims it
/// checks are those of [`SupabaseClaims::check`]. It keeps the keys it got at creation.
#[derive(Debug)]
pub struct SupabaseTokenVerifier {
    issuer: String,
    keys: HashMap<String, SigningKey>,
}

/// A public key of the project and the one algorithm it checks.
#[derive(Debug)]
struct SigningKey {
    algorithm: Algorithm,
    decoding_key: DecodingKey,
}

impl SupabaseTokenVerifier {
    /// Fetches the public keys of the Supabase project at `supabase_url` and creates the verifier.
    ///
    /// `supabase_url` is the base URL of the project, such as `http://127.0.0.1:54321` for the
    /// local stack. The keys come from `<supabase_url>/auth/v1/.well-known/jwks.json`.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::FailedKeyFetch`] if the request fails, the server answers with an
    /// error status, or the body is not a key set that [`Self::from_jwk_set`] accepts.
    pub async fn fetch(client: &reqwest::Client, supabase_url: &str) -> Result<Self, TokenError> {
        let base_url = supabase_url.trim_end_matches('/');
        let jwk_set = client
            .get(format!("{base_url}{JWKS_PATH}"))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| failed_key_fetch(&error))?
            .json::<JwkSet>()
            .await
            .map_err(|error| failed_key_fetch(&error))?;
        Self::from_jwk_set(base_url, &jwk_set)
    }

    /// Creates the verifier from a key set of the Supabase project at `supabase_url`.
    ///
    /// It keeps every key that has a key ID and that signs `ES256` or `RS256`. It skips every other
    /// key, such as a shared `HS256` secret or a key for encryption.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::FailedKeyFetch`] if a kept key does not decode.
    pub fn from_jwk_set(supabase_url: &str, jwk_set: &JwkSet) -> Result<Self, TokenError> {
        let mut keys = HashMap::new();
        for jwk in &jwk_set.keys {
            let (Some(kid), Some(algorithm)) = (&jwk.common.key_id, signing_algorithm(jwk)) else {
                continue;
            };
            let decoding_key =
                DecodingKey::from_jwk(jwk).map_err(|error| failed_key_fetch(&error))?;
            keys.insert(
                kid.clone(),
                SigningKey {
                    algorithm,
                    decoding_key,
                },
            );
        }
        Ok(Self {
            issuer: format!("{}{AUTH_PATH}", supabase_url.trim_end_matches('/')),
            keys,
        })
    }

    fn check(&self, token: &str) -> Result<VerifiedToken, TokenError> {
        let header =
            jsonwebtoken::decode_header(token).map_err(|error| invalid_token_format(&error))?;
        if !matches!(header.alg, Algorithm::ES256 | Algorithm::RS256) {
            return Err(TokenError::InvalidTokenFormat {
                reason: format!("the algorithm '{:?}' is not accepted", header.alg),
            });
        }
        let kid = header.kid.ok_or_else(|| TokenError::InvalidTokenFormat {
            reason: "the header names no key".to_owned(),
        })?;
        let key = self
            .keys
            .get(&kid)
            .ok_or(TokenError::UnknownSigningKey { kid })?;
        let claims = jsonwebtoken::decode::<SupabaseClaims>(
            token,
            &key.decoding_key,
            &signature_only(key.algorithm),
        )
        .map_err(|error| invalid_signature_or_format(&error))?
        .claims;
        claims.check(&self.issuer, Utc::now())
    }
}

#[async_trait]
impl TokenVerifier for SupabaseTokenVerifier {
    async fn verify(&self, token: &str) -> Result<VerifiedToken, TokenError> {
        self.check(token)
    }
}

/// Returns the algorithm that `jwk` signs, if it is a signing key for `ES256` or `RS256`.
fn signing_algorithm(jwk: &Jwk) -> Option<Algorithm> {
    if matches!(jwk.common.public_key_use, Some(ref key_use) if *key_use != PublicKeyUse::Signature)
    {
        return None;
    }
    match (jwk.common.key_algorithm, &jwk.algorithm) {
        (Some(KeyAlgorithm::ES256) | None, AlgorithmParameters::EllipticCurve(parameters)) => {
            (parameters.curve == EllipticCurve::P256).then_some(Algorithm::ES256)
        }
        (Some(KeyAlgorithm::RS256) | None, AlgorithmParameters::RSA(_)) => Some(Algorithm::RS256),
        _ => None,
    }
}

/// Builds a [`Validation`] that checks only the signature with `algorithm`.
///
/// [`SupabaseClaims::check`] checks the claims, so each failed claim names itself.
fn signature_only(algorithm: Algorithm) -> Validation {
    let mut validation = Validation::new(algorithm);
    validation.validate_exp = false;
    validation.validate_aud = false;
    validation.required_spec_claims.clear();
    validation
}

/// Maps a decode error: a signature that fails, or a key of the wrong family, is
/// [`TokenError::InvalidSignature`]. Every other error is [`TokenError::InvalidTokenFormat`].
fn invalid_signature_or_format(error: &JwtError) -> TokenError {
    match error.kind() {
        ErrorKind::InvalidSignature
        | ErrorKind::InvalidAlgorithm
        | ErrorKind::InvalidEcdsaKey
        | ErrorKind::InvalidRsaKey(_) => TokenError::InvalidSignature,
        _ => invalid_token_format(error),
    }
}

fn invalid_token_format(error: &JwtError) -> TokenError {
    let reason = match error.kind() {
        ErrorKind::InvalidToken => "the token does not have three parts".to_owned(),
        ErrorKind::Base64(_) => "a part is not base64url".to_owned(),
        ErrorKind::Json(_) => "a part is not the expected JSON".to_owned(),
        ErrorKind::Utf8(_) => "a part is not UTF-8".to_owned(),
        _ => error.to_string(),
    };
    TokenError::InvalidTokenFormat { reason }
}

fn failed_key_fetch(error: &impl Display) -> TokenError {
    TokenError::FailedKeyFetch {
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone};
    use jsonwebtoken::{EncodingKey, Header};
    use pretty_assertions::assert_eq;
    use serde_json::{Value, json};
    use storage::{AccountId, InvalidAccountId};
    use uuid::Uuid;

    use super::*;
    use crate::tests::fixtures::test_signing_key::TestSigningKey;

    const SUPABASE_URL: &str = "http://127.0.0.1:54321";
    const SUB: &str = "0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d";
    /// 2100-01-01 00:00:00 UTC, far enough ahead that the token never expires in a test.
    const EXP: i64 = 4_102_444_800;

    fn claims() -> Value {
        json!({
            "iss": "http://127.0.0.1:54321/auth/v1",
            "aud": "authenticated",
            "exp": EXP,
            "role": "authenticated",
            "is_anonymous": false,
            "sub": SUB,
            "session_id": "session-1",
        })
    }

    fn verified_token() -> VerifiedToken {
        VerifiedToken {
            account_id: AccountId::from_uuid(Uuid::from_u128(
                0x0193_b2c1_7d4e_7a2b_9c3d_4e5f_6a7b_8c9d,
            )),
            session_id: "session-1".to_owned(),
            expires_at: Utc
                .with_ymd_and_hms(2100, 1, 1, 0, 0, 0)
                .single()
                .expect("Given a valid hardcoded UTC time, building it should always succeed"),
        }
    }

    fn verifier(keys: &[&TestSigningKey]) -> SupabaseTokenVerifier {
        let jwk_set = JwkSet {
            keys: keys.iter().map(|key| key.jwk()).collect(),
        };
        SupabaseTokenVerifier::from_jwk_set(SUPABASE_URL, &jwk_set).expect(
            "Given the public keys of generated test keys, decoding them should always succeed",
        )
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_supabase_token_verifier() {
        implements_auto_traits::<SupabaseTokenVerifier>();
    }

    #[tokio::test]
    async fn should_return_the_verified_token_when_an_es256_key_signed_the_token() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&key]);
        let token = key.sign(&claims());

        let expected_result = Ok(verified_token());

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_verified_token_when_an_rs256_key_signed_the_token() {
        let key = TestSigningKey::rs256("key-1");
        let verifier = verifier(&[&key]);
        let token = key.sign(&claims());

        let expected_result = Ok(verified_token());

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_signature_when_another_key_signed_the_token_under_a_known_key_id()
     {
        let known_key = TestSigningKey::es256("key-1");
        let forging_key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&known_key]);
        let mut forged_claims = claims();
        forged_claims["sub"] = json!("not-a-uuid");
        let token = forging_key.sign(&forged_claims);

        let expected_result = Err(TokenError::InvalidSignature);

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_subject_when_a_known_key_signed_a_subject_that_is_not_a_uuid() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&key]);
        let mut token_claims = claims();
        token_claims["sub"] = json!("not-a-uuid");
        let token = key.sign(&token_claims);

        let expected_result = Err(TokenError::InvalidSubject {
            source: InvalidAccountId::new("not-a-uuid"),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_expired_token_when_the_token_expired() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&key]);
        let mut token_claims = claims();
        token_claims["exp"] = json!(1_700_000_000);
        let token = key.sign(&token_claims);

        let expected_result = Err(TokenError::ExpiredToken {
            expired_at: DateTime::from_timestamp(1_700_000_000, 0)
                .expect("Given a hardcoded time in range, building it should always succeed"),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_claim_when_the_audience_is_wrong() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&key]);
        let mut token_claims = claims();
        token_claims["aud"] = json!("anon");
        let token = key.sign(&token_claims);

        let expected_result = Err(TokenError::InvalidClaim {
            claim: "aud".to_owned(),
            reason: "the audience lacks 'authenticated'".to_owned(),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unknown_signing_key_when_the_key_id_is_unknown() {
        let known_key = TestSigningKey::es256("key-1");
        let other_key = TestSigningKey::es256("key-2");
        let verifier = verifier(&[&known_key]);
        let token = other_key.sign(&claims());

        let expected_result = Err(TokenError::UnknownSigningKey {
            kid: "key-2".to_owned(),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_signature_when_the_algorithm_does_not_match_the_key() {
        let ec_key = TestSigningKey::es256("key-1");
        let rsa_key = TestSigningKey::rs256("key-1");
        let verifier = verifier(&[&ec_key]);
        let token = rsa_key.sign(&claims());

        let expected_result = Err(TokenError::InvalidSignature);

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_token_format_when_the_token_uses_hs256() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&key]);
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some("key-1".to_owned());
        let secret = EncodingKey::from_secret(b"a-shared-secret-of-32-bytes-long")
            .expect("Given a hardcoded shared secret, building the key should always succeed");
        let token = jsonwebtoken::encode(&header, &claims(), &secret)
            .expect("Given a shared secret and JSON claims, signing should always succeed");

        let expected_result = Err(TokenError::InvalidTokenFormat {
            reason: "the algorithm 'HS256' is not accepted".to_owned(),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_token_format_when_the_header_names_no_key() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&key]);
        let token = key.sign_without_key_id(&claims());

        let expected_result = Err(TokenError::InvalidTokenFormat {
            reason: "the header names no key".to_owned(),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_token_format_when_the_token_is_not_a_jwt() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&key]);
        let token = "not-a-jwt";

        let expected_result = Err(TokenError::InvalidTokenFormat {
            reason: "the token does not have three parts".to_owned(),
        });

        let result = verifier.verify(token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unknown_signing_key_when_the_key_set_holds_only_a_shared_secret() {
        let key = TestSigningKey::es256("key-1");
        let secret: Jwk = serde_json::from_value(json!({
            "kty": "oct", "kid": "key-1", "alg": "HS256", "k": "c2VjcmV0",
        }))
        .expect("Given a hardcoded symmetric JWK, parsing it should always succeed");
        let verifier = SupabaseTokenVerifier::from_jwk_set(SUPABASE_URL, &JwkSet { keys: vec![secret] })
            .expect("Given a key set whose only key is skipped, creating the verifier should always succeed");
        let token = key.sign(&claims());

        let expected_result = Err(TokenError::UnknownSigningKey {
            kid: "key-1".to_owned(),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }
}
