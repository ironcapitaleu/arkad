//! # Fake Token Verifier
//!
//! Provides [`FakeTokenVerifier`], an in-memory [`TokenVerifier`] test double that accepts the
//! tokens a test sets up.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::traits::token_verifier::{TokenError, TokenVerifier, VerifiedToken};

/// An in-memory [`TokenVerifier`] test double.
///
/// It returns the [`VerifiedToken`] that a test set up for a token, and
/// [`TokenError::InvalidSignature`] for every other token. Built with
/// [`FakeTokenVerifier::failing`], it returns one error for every token.
#[derive(Debug, Default)]
pub struct FakeTokenVerifier {
    tokens: HashMap<String, VerifiedToken>,
    failure: Option<TokenError>,
}

impl FakeTokenVerifier {
    /// Returns the [`FakeTokenVerifier`] that also accepts `token` as `verified`.
    #[must_use]
    pub fn accepting(mut self, token: impl Into<String>, verified: VerifiedToken) -> Self {
        self.tokens.insert(token.into(), verified);
        self
    }

    /// Creates a [`FakeTokenVerifier`] that returns `error` for every token.
    ///
    /// It ignores the tokens that [`accepting`](Self::accepting) adds later.
    #[must_use]
    pub fn failing(error: TokenError) -> Self {
        Self {
            tokens: HashMap::new(),
            failure: Some(error),
        }
    }
}

#[async_trait]
impl TokenVerifier for FakeTokenVerifier {
    async fn verify(&self, token: &str) -> Result<VerifiedToken, TokenError> {
        if let Some(failure) = self.failure.clone() {
            return Err(failure);
        }
        self.tokens
            .get(token)
            .cloned()
            .ok_or(TokenError::InvalidSignature)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_fake_token_verifier() {
        implements_auto_traits::<FakeTokenVerifier>();
    }

    const fn implements_debug_and_default<T: Debug + Default>() {}
    #[test]
    const fn should_implement_debug_and_default_for_fake_token_verifier() {
        implements_debug_and_default::<FakeTokenVerifier>();
    }
}
