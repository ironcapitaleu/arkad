//! # Supabase Token Verifier
//!
//! Provides [`SupabaseTokenVerifier`], the [`TokenVerifier`] that checks Supabase login tokens
//! against the public keys of a Supabase project.
//!
//! ## Modules
//!
//! - [`jwks_source`]: The [`JwksSource`] that the verifier gets the public keys from.
//! - [`supabase_claims`]: The [`SupabaseClaims`] of a login token and their checks.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::errors::{Error as JwtError, ErrorKind};
use jsonwebtoken::jwk::{
    AlgorithmParameters, EllipticCurve, Jwk, JwkSet, KeyAlgorithm, PublicKeyUse,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde_json::json;
use tokio::task::JoinHandle;
use tokio::time::{Instant, MissedTickBehavior};

use crate::logging::ApiEvent;
use crate::traits::token_verifier::{TokenError, TokenVerifier, VerifiedToken};

pub mod jwks_source;
pub mod supabase_claims;

pub use jwks_source::{HttpJwksSource, JwksSource};
pub use supabase_claims::SupabaseClaims;

/// The path of the public keys below the URL of a Supabase project.
const JWKS_PATH: &str = "/auth/v1/.well-known/jwks.json";

/// The path of Supabase Auth below the URL of a Supabase project, which is the token issuer.
const AUTH_PATH: &str = "/auth/v1";

/// How often the task of [`SupabaseTokenVerifier::spawn_refresh`] refreshes the keys. It matches
/// the edge cache of the JWKS endpoint at Supabase.
pub const REFRESH_INTERVAL: Duration = Duration::from_mins(10);

/// The shortest time between two refreshes that a token with an unknown key ID starts.
pub const MIN_ON_DEMAND_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

/// A [`TokenVerifier`] that checks a Supabase login token with the public keys of the project.
///
/// It accepts tokens that an `ES256` or an `RS256` key signs, and rejects every other algorithm,
/// such as `HS256` and `none`. It checks the signature before it reads any claim. The claims it
/// checks are those of [`SupabaseClaims::check`].
///
/// It gets the keys at creation and keeps them in memory. Three events refresh them:
///
/// - The task of [`Self::spawn_refresh`], every [`REFRESH_INTERVAL`].
/// - A token with an unknown key ID, at most once per [`MIN_ON_DEMAND_REFRESH_INTERVAL`]. The
///   verifier then checks the token again.
/// - A call of [`Self::refresh`].
///
/// If a refresh fails, the verifier keeps the keys it has.
#[derive(Debug)]
pub struct SupabaseTokenVerifier {
    issuer: String,
    source: Box<dyn JwksSource>,
    keys: RwLock<HashMap<String, SigningKey>>,
    last_on_demand_refresh: Mutex<Option<Instant>>,
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
    /// error status, or a signing key in the body does not decode.
    pub async fn fetch(client: &reqwest::Client, supabase_url: &str) -> Result<Self, TokenError> {
        let base_url = supabase_url.trim_end_matches('/');
        let source = HttpJwksSource::new(client.clone(), format!("{base_url}{JWKS_PATH}"));
        Self::with_source(Box::new(source), base_url).await
    }

    /// Creates the verifier with the keys of `source`, for the project at `supabase_url`.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::FailedKeyFetch`] if `source` fails, or a signing key does not decode.
    pub async fn with_source(
        source: Box<dyn JwksSource>,
        supabase_url: &str,
    ) -> Result<Self, TokenError> {
        let keys = signing_keys(&source.fetch().await?)?;
        Ok(Self {
            issuer: format!("{}{AUTH_PATH}", supabase_url.trim_end_matches('/')),
            source,
            keys: RwLock::new(keys),
            last_on_demand_refresh: Mutex::new(None),
        })
    }

    /// Fetches the keys again and replaces the keys the verifier has.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::FailedKeyFetch`] if the source fails, or a signing key does not
    /// decode. The verifier then keeps the keys it has.
    pub async fn refresh(&self) -> Result<(), TokenError> {
        let keys = signing_keys(&self.source.fetch().await?)?;
        *self.keys.write().unwrap_or_else(PoisonError::into_inner) = keys;
        Ok(())
    }

    /// Spawns the task that refreshes the keys every [`REFRESH_INTERVAL`].
    ///
    /// The task logs a failed refresh as a `warn` event and tries again at the next interval. It
    /// ends when the last [`Arc`] of the verifier drops.
    pub fn spawn_refresh(verifier: &Arc<Self>) -> JoinHandle<()> {
        let verifier = Arc::downgrade(verifier);
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval_at(Instant::now() + REFRESH_INTERVAL, REFRESH_INTERVAL);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
            loop {
                interval.tick().await;
                let Some(verifier) = verifier.upgrade() else {
                    break;
                };
                verifier.refresh_or_warn().await;
            }
        })
    }

    async fn refresh_or_warn(&self) {
        if let Err(error) = self.refresh().await {
            tracing::warn!(
                event = %ApiEvent::SigningKeyRefreshFailed,
                message = %format!(
                    "[FailedKeyRefresh] Failed to refresh the signing keys, Caused by: {error}"
                ),
                context = %json!({ "issuer": self.issuer }),
            );
        }
    }

    /// Returns whether a token with an unknown key ID can start a refresh now, and records the
    /// start if so.
    fn claim_on_demand_refresh(&self) -> bool {
        let mut last = self
            .last_on_demand_refresh
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let now = Instant::now();
        if last.is_some_and(|at| now.duration_since(at) < MIN_ON_DEMAND_REFRESH_INTERVAL) {
            return false;
        }
        *last = Some(now);
        true
    }

    fn check(&self, token: &str) -> Result<VerifiedToken, TokenError> {
        let header =
            jsonwebtoken::decode_header(token).map_err(|error| invalid_token_format(&error))?;
        if !matches!(header.alg, Algorithm::ES256 | Algorithm::RS256) {
            return Err(TokenError::InvalidTokenFormat {
                reason: "the algorithm is not ES256 or RS256".to_owned(),
            });
        }
        let kid = header.kid.ok_or_else(|| TokenError::InvalidTokenFormat {
            reason: "the header names no key".to_owned(),
        })?;
        let keys = self.keys.read().unwrap_or_else(PoisonError::into_inner);
        let decoded = keys.get(&kid).map(|key| {
            jsonwebtoken::decode::<SupabaseClaims>(
                token,
                &key.decoding_key,
                &signature_only(key.algorithm),
            )
        });
        drop(keys);
        let claims = decoded
            .ok_or(TokenError::UnknownSigningKey { kid })?
            .map_err(|error| invalid_signature_or_format(&error))?
            .claims;
        claims.check(&self.issuer, Utc::now())
    }
}

#[async_trait]
impl TokenVerifier for SupabaseTokenVerifier {
    async fn verify(&self, token: &str) -> Result<VerifiedToken, TokenError> {
        match self.check(token) {
            Err(TokenError::UnknownSigningKey { .. }) if self.claim_on_demand_refresh() => {
                self.refresh_or_warn().await;
                self.check(token)
            }
            result => result,
        }
    }
}

/// Keeps every key of `jwk_set` that has a key ID and that signs `ES256` or `RS256`.
///
/// It skips every other key, such as a shared `HS256` secret or a key for encryption. If two kept
/// keys share a key ID, the first one wins.
fn signing_keys(jwk_set: &JwkSet) -> Result<HashMap<String, SigningKey>, TokenError> {
    let mut keys = HashMap::new();
    for jwk in &jwk_set.keys {
        let (Some(kid), Some(algorithm)) = (&jwk.common.key_id, signing_algorithm(jwk)) else {
            continue;
        };
        if keys.contains_key(kid) {
            continue;
        }
        let decoding_key = DecodingKey::from_jwk(jwk).map_err(|_| TokenError::FailedKeyFetch {
            reason: format!("the key '{kid}' does not decode"),
        })?;
        keys.insert(
            kid.clone(),
            SigningKey {
                algorithm,
                decoding_key,
            },
        );
    }
    Ok(keys)
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
        _ => "the token does not decode".to_owned(),
    };
    TokenError::InvalidTokenFormat { reason }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use chrono::{DateTime, TimeZone};
    use jsonwebtoken::{EncodingKey, Header};
    use pretty_assertions::assert_eq;
    use serde_json::{Value, json};
    use storage::{AccountId, InvalidAccountId};
    use uuid::Uuid;

    use super::*;
    use crate::tests::fixtures::fake_jwks_source::FakeJwksSource;
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

    fn key_set(keys: &[&TestSigningKey]) -> JwkSet {
        JwkSet {
            keys: keys.iter().map(|key| key.jwk()).collect(),
        }
    }

    async fn verifier_with(source: FakeJwksSource) -> SupabaseTokenVerifier {
        SupabaseTokenVerifier::with_source(Box::new(source), SUPABASE_URL)
            .await
            .expect("Given a source with decodable test keys, creating the verifier should always succeed")
    }

    async fn verifier(keys: &[&TestSigningKey]) -> SupabaseTokenVerifier {
        verifier_with(FakeJwksSource::answering(vec![Ok(key_set(keys))])).await
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_supabase_token_verifier() {
        implements_auto_traits::<SupabaseTokenVerifier>();
    }

    #[tokio::test]
    async fn should_return_the_verified_token_when_an_es256_key_signed_the_token() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&key]).await;
        let token = key.sign(&claims());

        let expected_result = Ok(verified_token());

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_verified_token_when_an_rs256_key_signed_the_token() {
        let key = TestSigningKey::rs256("key-1");
        let verifier = verifier(&[&key]).await;
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
        let verifier = verifier(&[&known_key]).await;
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
        let verifier = verifier(&[&key]).await;
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
        let verifier = verifier(&[&key]).await;
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
        let verifier = verifier(&[&key]).await;
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
        let verifier = verifier(&[&known_key]).await;
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
        let verifier = verifier(&[&ec_key]).await;
        let token = rsa_key.sign(&claims());

        let expected_result = Err(TokenError::InvalidSignature);

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_token_format_when_the_token_uses_hs256() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&key]).await;
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some("key-1".to_owned());
        let secret = EncodingKey::from_secret(b"a-shared-secret-of-32-bytes-long")
            .expect("Given a hardcoded shared secret, building the key should always succeed");
        let token = jsonwebtoken::encode(&header, &claims(), &secret)
            .expect("Given a shared secret and JSON claims, signing should always succeed");

        let expected_result = Err(TokenError::InvalidTokenFormat {
            reason: "the algorithm is not ES256 or RS256".to_owned(),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_token_format_when_the_header_names_no_key() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&key]).await;
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
        let verifier = verifier(&[&key]).await;
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
        let verifier = verifier_with(FakeJwksSource::answering(vec![Ok(JwkSet {
            keys: vec![secret],
        })]))
        .await;
        let token = key.sign(&claims());

        let expected_result = Err(TokenError::UnknownSigningKey {
            kid: "key-1".to_owned(),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    fn failed_key_fetch_error() -> TokenError {
        TokenError::FailedKeyFetch {
            reason: "the JWKS endpoint answered 503".to_owned(),
        }
    }

    #[tokio::test(start_paused = true)]
    async fn should_return_the_verified_token_when_a_refresh_brings_the_unknown_key() {
        let old_key = TestSigningKey::es256("key-1");
        let new_key = TestSigningKey::es256("key-2");
        let source = FakeJwksSource::answering(vec![
            Ok(key_set(&[&old_key])),
            Ok(key_set(&[&old_key, &new_key])),
        ]);
        let verifier = verifier_with(source).await;
        let token = new_key.sign(&claims());

        let expected_result = Ok(verified_token());

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test(start_paused = true)]
    async fn should_fetch_no_keys_when_a_second_unknown_key_id_comes_within_30_seconds() {
        let key = TestSigningKey::es256("key-1");
        let unknown_key = TestSigningKey::es256("key-2");
        let source = FakeJwksSource::answering(vec![Ok(key_set(&[&key]))]);
        let fetches = source.fetches();
        let verifier = verifier_with(source).await;
        let token = unknown_key.sign(&claims());
        let _first = verifier.verify(&token).await;
        tokio::time::advance(Duration::from_secs(29)).await;

        let expected_result = 2;

        let _second = verifier.verify(&token).await;
        let result = fetches.load(Ordering::SeqCst);

        assert_eq!(result, expected_result);
    }

    #[tokio::test(start_paused = true)]
    async fn should_fetch_the_keys_again_when_a_second_unknown_key_id_comes_after_30_seconds() {
        let key = TestSigningKey::es256("key-1");
        let unknown_key = TestSigningKey::es256("key-2");
        let source = FakeJwksSource::answering(vec![Ok(key_set(&[&key]))]);
        let fetches = source.fetches();
        let verifier = verifier_with(source).await;
        let token = unknown_key.sign(&claims());
        let _first = verifier.verify(&token).await;
        tokio::time::advance(MIN_ON_DEMAND_REFRESH_INTERVAL).await;

        let expected_result = 3;

        let _second = verifier.verify(&token).await;
        let result = fetches.load(Ordering::SeqCst);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_key_fetch_when_a_refresh_fails() {
        let key = TestSigningKey::es256("key-1");
        let source =
            FakeJwksSource::answering(vec![Ok(key_set(&[&key])), Err(failed_key_fetch_error())]);
        let verifier = verifier_with(source).await;

        let expected_result = Err(failed_key_fetch_error());

        let result = verifier.refresh().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_old_keys_when_a_refresh_fails() {
        let key = TestSigningKey::es256("key-1");
        let source =
            FakeJwksSource::answering(vec![Ok(key_set(&[&key])), Err(failed_key_fetch_error())]);
        let verifier = verifier_with(source).await;
        let _refresh = verifier.refresh().await;
        let token = key.sign(&claims());

        let expected_result = Ok(verified_token());

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test(start_paused = true)]
    async fn should_fetch_the_keys_once_per_interval_when_the_refresh_task_runs() {
        let key = TestSigningKey::es256("key-1");
        let source = FakeJwksSource::answering(vec![Ok(key_set(&[&key]))]);
        let fetches = source.fetches();
        let verifier = Arc::new(verifier_with(source).await);
        let _task = SupabaseTokenVerifier::spawn_refresh(&verifier);

        let expected_result = 3;

        tokio::time::sleep(REFRESH_INTERVAL * 2 + Duration::from_secs(1)).await;
        let result = fetches.load(Ordering::SeqCst);

        assert_eq!(result, expected_result);
    }

    #[tokio::test(start_paused = true)]
    async fn should_end_the_refresh_task_when_the_verifier_drops() {
        let key = TestSigningKey::es256("key-1");
        let verifier = Arc::new(verifier(&[&key]).await);
        let task = SupabaseTokenVerifier::spawn_refresh(&verifier);
        drop(verifier);

        let expected_result = true;

        tokio::time::sleep(REFRESH_INTERVAL + Duration::from_secs(1)).await;
        let result = task.is_finished();

        assert_eq!(result, expected_result);
    }

    /// Returns the public JWK of `key` with `field` set to `value`, or removed if `value` is null.
    fn jwk_with(key: &TestSigningKey, field: &str, value: Value) -> Jwk {
        let mut jwk = serde_json::to_value(key.jwk())
            .expect("Given a JWK of a generated key, serializing it should always succeed");
        if value.is_null() {
            jwk.as_object_mut()
                .map(|object| object.remove(field))
                .expect("Given a serialized JWK, it should always be a JSON object");
        } else {
            jwk[field] = value;
        }
        serde_json::from_value(jwk)
            .expect("Given a JWK with one changed field, parsing it should always succeed")
    }

    async fn verifier_from_jwk(jwk: Jwk) -> SupabaseTokenVerifier {
        verifier_with(FakeJwksSource::answering(vec![Ok(JwkSet {
            keys: vec![jwk],
        })]))
        .await
    }

    #[tokio::test]
    async fn should_return_unknown_signing_key_when_the_only_key_is_for_encryption() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier_from_jwk(jwk_with(&key, "use", json!("enc"))).await;
        let token = key.sign(&claims());

        let expected_result = Err(TokenError::UnknownSigningKey {
            kid: "key-1".to_owned(),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unknown_signing_key_when_the_only_key_is_on_another_curve() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier_from_jwk(jwk_with(&key, "crv", json!("P-384"))).await;
        let token = key.sign(&claims());

        let expected_result = Err(TokenError::UnknownSigningKey {
            kid: "key-1".to_owned(),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unknown_signing_key_when_the_only_key_names_another_algorithm() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier_from_jwk(jwk_with(&key, "alg", json!("RS256"))).await;
        let token = key.sign(&claims());

        let expected_result = Err(TokenError::UnknownSigningKey {
            kid: "key-1".to_owned(),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unknown_signing_key_when_the_only_key_has_no_key_id() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier_from_jwk(jwk_with(&key, "kid", Value::Null)).await;
        let token = key.sign(&claims());

        let expected_result = Err(TokenError::UnknownSigningKey {
            kid: "key-1".to_owned(),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_verified_token_when_the_first_of_two_keys_with_one_key_id_signed() {
        let first_key = TestSigningKey::es256("key-1");
        let second_key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&first_key, &second_key]).await;
        let token = first_key.sign(&claims());

        let expected_result = Ok(verified_token());

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_token_format_when_the_token_uses_alg_none() {
        let key = TestSigningKey::es256("key-1");
        let verifier = verifier(&[&key]).await;
        // `{"alg":"none","kid":"key-1"}`, the claims, and an empty signature.
        let token = format!(
            "eyJhbGciOiJub25lIiwia2lkIjoia2V5LTEifQ.{}.",
            URL_SAFE_NO_PAD.encode(claims().to_string())
        );

        let expected_result = Err(TokenError::InvalidTokenFormat {
            reason: "a part is not the expected JSON".to_owned(),
        });

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_verified_token_when_the_fetched_key_signed_the_token() {
        let key = TestSigningKey::es256("key-1");
        let jwk_set = JwkSet {
            keys: vec![key.jwk()],
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Given a free port on the loopback interface, binding should always succeed");
        let port = listener
            .local_addr()
            .expect("Given a bound listener, reading its address should always succeed")
            .port();
        let app = axum::Router::new().route(
            JWKS_PATH,
            axum::routing::get(move || async move { axum::Json(jwk_set) }),
        );
        tokio::spawn(async move { axum::serve(listener, app).await });
        let verifier = SupabaseTokenVerifier::fetch(
            &reqwest::Client::new(),
            &format!("http://127.0.0.1:{port}/"),
        )
        .await
        .expect("Given a server that serves a key set, fetching it should always succeed");
        let mut token_claims = claims();
        token_claims["iss"] = json!(format!("http://127.0.0.1:{port}/auth/v1"));
        let token = key.sign(&token_claims);

        let expected_result = Ok(verified_token());

        let result = verifier.verify(&token).await;

        assert_eq!(result, expected_result);
    }
}
