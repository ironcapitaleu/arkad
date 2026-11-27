//! # Supabase Claims
//!
//! Provides [`SupabaseClaims`], the claims of a Supabase login token, and their checks.

use chrono::{DateTime, TimeDelta, Utc};
use serde::Deserialize;
use storage::AccountId;

use crate::traits::token_verifier::{TokenError, VerifiedToken};

/// How long after its `exp` time a token still passes, to allow for clock skew.
const EXPIRY_LEEWAY: TimeDelta = TimeDelta::seconds(30);

/// The audience and the role of a token for a signed-in user.
const AUTHENTICATED: &str = "authenticated";

/// The claims of a Supabase login token that the checks read.
///
/// Every field is optional, so a missing claim fails its own check with a named claim.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SupabaseClaims {
    /// The issuer, `<supabase url>/auth/v1`.
    pub iss: Option<String>,
    /// The audience, one string or a list.
    pub aud: Option<Audience>,
    /// The expiry, in seconds since the Unix epoch.
    pub exp: Option<i64>,
    /// The Postgres role of the token.
    pub role: Option<String>,
    /// Whether the token belongs to an anonymous sign-in.
    pub is_anonymous: Option<bool>,
    /// The user ID of Supabase Auth.
    pub sub: Option<String>,
    /// The Supabase session of the token.
    pub session_id: Option<String>,
}

/// The `aud` claim, which a JWT holds as one string or as a list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Audience {
    /// One audience.
    One(String),
    /// Several audiences.
    Many(Vec<String>),
}

impl Audience {
    fn contains(&self, value: &str) -> bool {
        match self {
            Self::One(audience) => audience == value,
            Self::Many(audiences) => audiences.iter().any(|audience| audience == value),
        }
    }
}

impl SupabaseClaims {
    /// Checks the claims of a token whose signature passed, and returns its facts.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::ExpiredToken`] if `now` is more than 30 seconds past `exp`.
    /// Returns [`TokenError::InvalidSubject`] if `sub` is not a UUID. Returns
    /// [`TokenError::InvalidClaim`] if a claim is missing, or if `iss` differs from `issuer`, `aud`
    /// lacks `authenticated`, `role` is not `authenticated`, or `is_anonymous` is not `false`.
    pub fn check(self, issuer: &str, now: DateTime<Utc>) -> Result<VerifiedToken, TokenError> {
        let expires_at = Self::check_expiry(self.exp, now)?;
        let iss = required("iss", self.iss)?;
        if iss != issuer {
            return Err(invalid_claim(
                "iss",
                format!("the issuer is not '{issuer}'"),
            ));
        }
        let aud = required("aud", self.aud)?;
        if !aud.contains(AUTHENTICATED) {
            return Err(invalid_claim("aud", "the audience lacks 'authenticated'"));
        }
        if required("role", self.role)? != AUTHENTICATED {
            return Err(invalid_claim("role", "the role is not 'authenticated'"));
        }
        if required("is_anonymous", self.is_anonymous)? {
            return Err(invalid_claim("is_anonymous", "the token is anonymous"));
        }
        let sub = required("sub", self.sub)?;
        let account_id =
            AccountId::parse(&sub).map_err(|source| TokenError::InvalidSubject { source })?;
        let session_id = required("session_id", self.session_id)?;
        Ok(VerifiedToken {
            account_id,
            session_id,
            expires_at,
        })
    }

    fn check_expiry(exp: Option<i64>, now: DateTime<Utc>) -> Result<DateTime<Utc>, TokenError> {
        let exp = required("exp", exp)?;
        let expires_at = DateTime::from_timestamp(exp, 0)
            .ok_or_else(|| invalid_claim("exp", "the time is out of range"))?;
        let deadline = expires_at
            .checked_add_signed(EXPIRY_LEEWAY)
            .ok_or_else(|| invalid_claim("exp", "the time is out of range"))?;
        if now > deadline {
            return Err(TokenError::ExpiredToken {
                expired_at: expires_at,
            });
        }
        Ok(expires_at)
    }
}

fn required<T>(claim: &str, value: Option<T>) -> Result<T, TokenError> {
    value.ok_or_else(|| invalid_claim(claim, "the claim is missing"))
}

fn invalid_claim(claim: &str, reason: impl Into<String>) -> TokenError {
    TokenError::InvalidClaim {
        claim: claim.to_owned(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;
    use storage::InvalidAccountId;
    use uuid::Uuid;

    use super::*;

    const ISSUER: &str = "http://127.0.0.1:54321/auth/v1";
    const SUB: &str = "0193b2c1-7d4e-7a2b-9c3d-4e5f6a7b8c9d";
    /// 2026-11-15 10:00:00 UTC.
    const EXP: i64 = 1_794_736_800;

    fn time(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 15, hour, minute, second)
            .single()
            .expect("Given a valid hardcoded UTC time, building it should always succeed")
    }

    fn valid_claims() -> SupabaseClaims {
        SupabaseClaims {
            iss: Some(ISSUER.to_owned()),
            aud: Some(Audience::One("authenticated".to_owned())),
            exp: Some(EXP),
            role: Some("authenticated".to_owned()),
            is_anonymous: Some(false),
            sub: Some(SUB.to_owned()),
            session_id: Some("session-1".to_owned()),
        }
    }

    fn verified_token() -> VerifiedToken {
        VerifiedToken {
            account_id: AccountId::from_uuid(Uuid::from_u128(
                0x0193_b2c1_7d4e_7a2b_9c3d_4e5f_6a7b_8c9d,
            )),
            session_id: "session-1".to_owned(),
            expires_at: time(10, 0, 0),
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_supabase_claims() {
        implements_auto_traits::<SupabaseClaims>();
    }

    #[test]
    fn should_return_the_verified_token_when_every_claim_passes() {
        let claims = valid_claims();
        let now = time(9, 30, 0);

        let expected_result = Ok(verified_token());

        let result = claims.check(ISSUER, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_verified_token_when_the_token_expired_within_the_leeway() {
        let claims = valid_claims();
        let now = time(10, 0, 30);

        let expected_result = Ok(verified_token());

        let result = claims.check(ISSUER, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_expired_token_when_the_token_expired_beyond_the_leeway() {
        let claims = valid_claims();
        let now = time(10, 0, 31);

        let expected_result = Err(TokenError::ExpiredToken {
            expired_at: time(10, 0, 0),
        });

        let result = claims.check(ISSUER, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_claim_when_the_issuer_differs() {
        let claims = SupabaseClaims {
            iss: Some("https://other.supabase.co/auth/v1".to_owned()),
            ..valid_claims()
        };
        let now = time(9, 30, 0);

        let expected_result = Err(TokenError::InvalidClaim {
            claim: "iss".to_owned(),
            reason: "the issuer is not 'http://127.0.0.1:54321/auth/v1'".to_owned(),
        });

        let result = claims.check(ISSUER, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_claim_when_the_audience_lacks_authenticated() {
        let claims = SupabaseClaims {
            aud: Some(Audience::Many(vec!["anon".to_owned()])),
            ..valid_claims()
        };
        let now = time(9, 30, 0);

        let expected_result = Err(TokenError::InvalidClaim {
            claim: "aud".to_owned(),
            reason: "the audience lacks 'authenticated'".to_owned(),
        });

        let result = claims.check(ISSUER, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_verified_token_when_the_audience_list_holds_authenticated() {
        let claims = SupabaseClaims {
            aud: Some(Audience::Many(vec![
                "other".to_owned(),
                "authenticated".to_owned(),
            ])),
            ..valid_claims()
        };
        let now = time(9, 30, 0);

        let expected_result = Ok(verified_token());

        let result = claims.check(ISSUER, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_claim_when_the_role_is_anon() {
        let claims = SupabaseClaims {
            role: Some("anon".to_owned()),
            ..valid_claims()
        };
        let now = time(9, 30, 0);

        let expected_result = Err(TokenError::InvalidClaim {
            claim: "role".to_owned(),
            reason: "the role is not 'authenticated'".to_owned(),
        });

        let result = claims.check(ISSUER, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_claim_when_the_token_is_anonymous() {
        let claims = SupabaseClaims {
            is_anonymous: Some(true),
            ..valid_claims()
        };
        let now = time(9, 30, 0);

        let expected_result = Err(TokenError::InvalidClaim {
            claim: "is_anonymous".to_owned(),
            reason: "the token is anonymous".to_owned(),
        });

        let result = claims.check(ISSUER, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_subject_when_the_subject_is_not_a_uuid() {
        let claims = SupabaseClaims {
            sub: Some("service-role".to_owned()),
            ..valid_claims()
        };
        let now = time(9, 30, 0);

        let expected_result = Err(TokenError::InvalidSubject {
            source: InvalidAccountId::new("service-role"),
        });

        let result = claims.check(ISSUER, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_claim_when_the_session_id_is_missing() {
        let claims = SupabaseClaims {
            session_id: None,
            ..valid_claims()
        };
        let now = time(9, 30, 0);

        let expected_result = Err(TokenError::InvalidClaim {
            claim: "session_id".to_owned(),
            reason: "the claim is missing".to_owned(),
        });

        let result = claims.check(ISSUER, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_claim_when_the_expiry_is_missing() {
        let claims = SupabaseClaims {
            exp: None,
            ..valid_claims()
        };
        let now = time(9, 30, 0);

        let expected_result = Err(TokenError::InvalidClaim {
            claim: "exp".to_owned(),
            reason: "the claim is missing".to_owned(),
        });

        let result = claims.check(ISSUER, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_claim_when_the_expiry_plus_the_leeway_is_out_of_range() {
        let claims = SupabaseClaims {
            exp: Some(8_210_266_876_799),
            ..valid_claims()
        };
        let now = time(9, 30, 0);

        let expected_result = Err(TokenError::InvalidClaim {
            claim: "exp".to_owned(),
            reason: "the time is out of range".to_owned(),
        });

        let result = claims.check(ISSUER, now);

        assert_eq!(result, expected_result);
    }
}
