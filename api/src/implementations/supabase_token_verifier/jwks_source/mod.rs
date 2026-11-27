//! # JWKS Source
//!
//! Provides [`JwksSource`], where [`SupabaseTokenVerifier`](super::SupabaseTokenVerifier) gets
//! the public keys of a project, and [`HttpJwksSource`], which fetches them over HTTP.

use std::fmt::{Debug, Display};

use async_trait::async_trait;
use jsonwebtoken::jwk::JwkSet;

use crate::traits::token_verifier::TokenError;

/// Fetches the current key set of a Supabase project.
#[async_trait]
pub trait JwksSource: Debug + Send + Sync {
    /// Fetches the current key set.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::FailedKeyFetch`] if the key set cannot be fetched.
    async fn fetch(&self) -> Result<JwkSet, TokenError>;
}

/// A [`JwksSource`] that fetches the key set from the JWKS endpoint of a project.
#[derive(Debug, Clone)]
pub struct HttpJwksSource {
    client: reqwest::Client,
    url: String,
}

impl HttpJwksSource {
    /// Creates a source that fetches the key set from `url` with `client`.
    #[must_use]
    pub const fn new(client: reqwest::Client, url: String) -> Self {
        Self { client, url }
    }
}

#[async_trait]
impl JwksSource for HttpJwksSource {
    async fn fetch(&self) -> Result<JwkSet, TokenError> {
        self.client
            .get(&self.url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| failed_key_fetch(&error))?
            .json::<JwkSet>()
            .await
            .map_err(|error| failed_key_fetch(&error))
    }
}

/// Wraps a failure to get or read the key set into [`TokenError::FailedKeyFetch`].
fn failed_key_fetch(error: &impl Display) -> TokenError {
    TokenError::FailedKeyFetch {
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_http_jwks_source() {
        implements_auto_traits::<HttpJwksSource>();
    }
}
