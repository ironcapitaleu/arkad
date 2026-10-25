//! # SEC Client
//!
//! Provides the [`SecClient`], the concrete [`SecClient`](crate::shared::http_client::SecClient)
//! used throughout the pipeline.
//!
//! Rate limiting is globally shared within this process: all [`SecClient`] instances — whether
//! constructed independently or cloned — draw from a single budget, ensuring the SEC's request-rate
//! ceiling is not exceeded by this process regardless of how many clients exist.
//!
//! ## Modules
//!
//! - [`error`]: The [`FailedSecRequest`] error returned when execution fails.

use std::sync::OnceLock;

use async_trait::async_trait;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::shared::http_client::InnerClient;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::rate_limiter::{RateLimiter, SecRateLimiter};
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::shared::user_agent::UserAgent;
use crate::shared::user_agent::constants::DEFAULT_SEC_USER_AGENT;

use self::error::FailedSecRequest;

pub mod error;

/// Shared rate limiter backing all [`SecClient`] instances.
static GLOBAL_RATE_LIMITER: OnceLock<SecRateLimiter> = OnceLock::new();

/// The default SEC API client, driving the full `SecRequest` → `SecResponse` cycle.
///
/// Executes a validated request through a `reqwest::Client` and validates the reply into a
/// [`SecResponse`].
///
/// # Rate Limiting
///
/// Every request first awaits a permit from the shared [`SecRateLimiter`], pacing outgoing traffic
/// under the SEC's request-rate ceiling. Multiple instances can maintain separate TLS connections
/// while still collectively respecting the rate limit.
///
/// # Cloning
///
/// `reqwest::Client` is `Arc`-backed, so clones share one connection pool, TLS sessions, and DNS
/// cache — no extra `Arc` wrapping is needed for concurrent use. To use separate connection pools
/// while retaining the shared rate limit, construct independent instances via [`SecClient::new`].
///
/// # User Agent
///
/// Every constructor sets the `User-Agent` from a validated [`UserAgent`], so every request
/// carries the SEC-compliant header the API requires. [`SecClient::user_agent`] returns that value.
#[derive(Debug, Clone)]
pub struct SecClient {
    inner: reqwest::Client,
    rate_limiter: SecRateLimiter,
    user_agent: UserAgent,
}

impl Serialize for SecClient {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let state = serializer.serialize_struct("SecClient", 0)?;
        state.end()
    }
}

impl SecClient {
    /// Creates a new [`SecClient`] that sends `user_agent` as its `User-Agent` header, backed by
    /// the shared [`SecRateLimiter`] that paces requests under the SEC's request-rate ceiling.
    ///
    /// # Panics
    ///
    /// Panics if `reqwest` cannot initialize its TLS backend.
    #[must_use]
    pub fn new(user_agent: UserAgent) -> Self {
        let inner = reqwest::Client::builder()
            .user_agent(user_agent.inner())
            .build()
            .expect("A validated UserAgent should always produce a valid HTTP client");
        Self {
            inner,
            rate_limiter: GLOBAL_RATE_LIMITER.get_or_init(SecRateLimiter::new).clone(),
            user_agent,
        }
    }

    /// Returns the [`UserAgent`] this client sends in its `User-Agent` header.
    #[must_use]
    pub const fn user_agent(&self) -> &UserAgent {
        &self.user_agent
    }
}

/// Creates a default [`SecClient`] configured with the default SEC user agent.
impl Default for SecClient {
    fn default() -> Self {
        let user_agent = UserAgent::new(DEFAULT_SEC_USER_AGENT)
            .expect("The default SEC user agent constant should always be valid");
        Self::new(user_agent)
    }
}

// Deviation: neither `reqwest::Client` nor the `SecRateLimiter` expose any comparable or
// hashable state, so these impls compare and hash only the `user_agent` field. This satisfies
// trait bounds that require `Eq + Ord + Hash` (e.g. for use in collections or state machines).

impl PartialEq for SecClient {
    fn eq(&self, other: &Self) -> bool {
        self.user_agent == other.user_agent
    }
}

impl Eq for SecClient {}

impl std::hash::Hash for SecClient {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.user_agent, state);
    }
}

impl PartialOrd for SecClient {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SecClient {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.user_agent.cmp(&other.user_agent)
    }
}

#[async_trait]
impl SecClientTrait for SecClient {
    type Inner = reqwest::Client;
    type Limiter = SecRateLimiter;
    type Request = SecRequest;
    type Response = SecResponse;
    type Error = FailedSecRequest;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn rate_limiter(&self) -> &Self::Limiter {
        &self.rate_limiter
    }

    async fn execute_sec_request(
        &self,
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        let inner_request = request.into_inner();

        self.rate_limiter().await_turn().await;

        let inner_response = self.inner.execute_request(inner_request).await?;
        let sec_response = SecResponse::from_inner(inner_response).await?;
        Ok(sec_response)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::SecClient;
    use crate::shared::user_agent::UserAgent;

    #[test]
    fn should_serialize_to_empty_struct_when_serialized_to_json() {
        let client = SecClient::default();

        let expected_result = serde_json::json!({});

        let result = serde_json::to_value(&client).expect("SecClient should serialize to JSON");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_given_user_agent_when_created_with_new() {
        let user_agent = UserAgent::new("Sample Company contact@example.com")
            .expect("A hardcoded SEC-format string should always validate");
        let client = SecClient::new(user_agent.clone());

        let expected_result = &user_agent;

        let result = client.user_agent();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_equal_default_client_when_user_agent_differs() {
        let user_agent = UserAgent::new("Sample Company contact@example.com")
            .expect("A hardcoded SEC-format string should always validate");
        let client = SecClient::new(user_agent);

        let expected_result = SecClient::default();

        let result = client;

        assert_ne!(result, expected_result);
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}
    const fn implements_unpin<T: Unpin>() {}

    #[test]
    const fn should_be_send_for_sec_client() {
        implements_send::<SecClient>();
    }

    #[test]
    const fn should_be_sync_for_sec_client() {
        implements_sync::<SecClient>();
    }

    #[test]
    const fn should_be_unpin_for_sec_client() {
        implements_unpin::<SecClient>();
    }
}
