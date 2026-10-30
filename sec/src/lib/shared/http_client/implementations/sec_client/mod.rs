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

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::rate_limiter::SecRateLimiter;
use crate::shared::response::implementations::sec_response::{ResponseBody, SecResponse};
use crate::shared::user_agent::UserAgent;

use self::error::FailedSecRequest;
use super::reqwest_client::ReqwestClient;

pub mod error;

/// Shared rate limiter backing all [`SecClient`] instances.
static GLOBAL_RATE_LIMITER: OnceLock<SecRateLimiter> = OnceLock::new();

/// The default SEC API client, driving the full `SecRequest` → `SecResponse` cycle.
///
/// Executes a validated request through a [`ReqwestClient`] and validates the reply into an
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
/// The [`ReqwestClient`] holds the [`UserAgent`] that every request carries.
/// [`SecClient::user_agent`] returns it, so other code reads the `User-Agent` from the client
/// that sends it. Two clients are equal if they send the same `User-Agent`.
#[derive(Debug, Clone)]
pub struct SecClient {
    inner: ReqwestClient,
    rate_limiter: SecRateLimiter,
}

// The `User-Agent` contains a contact email. It is not sensitive, so the client serializes it.
impl Serialize for SecClient {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("SecClient", 1)?;
        state.serialize_field("user_agent", self.user_agent())?;
        state.end()
    }
}

impl SecClient {
    /// Creates a new [`SecClient`] with the given [`ReqwestClient`], backed by the shared
    /// [`SecRateLimiter`] that paces requests under the SEC's request-rate ceiling.
    #[must_use]
    pub fn new(inner: ReqwestClient) -> Self {
        Self {
            inner,
            rate_limiter: GLOBAL_RATE_LIMITER.get_or_init(SecRateLimiter::new).clone(),
        }
    }

    /// Returns the [`UserAgent`] that every request carries.
    #[must_use]
    pub const fn user_agent(&self) -> &UserAgent {
        self.inner.user_agent()
    }
}

impl Default for SecClient {
    /// Returns an [`SecClient`] that sends the default [`UserAgent`].
    ///
    /// # Panics
    ///
    /// Panics if `reqwest` cannot build the HTTP client, for example when the TLS backend fails
    /// to initialize.
    fn default() -> Self {
        let inner = ReqwestClient::new(UserAgent::default())
            .expect("Given the validated default user agent, building the HTTP client fails only if the TLS backend cannot initialize");
        Self::new(inner)
    }
}

// Deviation: neither `reqwest::Client` nor the `SecRateLimiter` expose comparable or hashable
// state. Equality, ordering and hashing use only the `UserAgent` the client sends. This satisfies
// trait bounds that require `Eq + Ord + Hash` (e.g. for use in collections or state machines).

impl PartialEq for SecClient {
    fn eq(&self, other: &Self) -> bool {
        self.user_agent() == other.user_agent()
    }
}

impl Eq for SecClient {}

impl std::hash::Hash for SecClient {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.user_agent().hash(state);
    }
}

impl PartialOrd for SecClient {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SecClient {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.user_agent().cmp(other.user_agent())
    }
}

impl SecClientTrait for SecClient {
    type Inner = ReqwestClient;
    type Limiter = SecRateLimiter;
    type Response<B: ResponseBody> = SecResponse<B>;
    type Error = FailedSecRequest;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn rate_limiter(&self) -> &Self::Limiter {
        &self.rate_limiter
    }

    fn user_agent(&self) -> &UserAgent {
        Self::user_agent(self)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::SecClient;
    use crate::shared::http_client::implementations::reqwest_client::ReqwestClient;
    use crate::shared::user_agent::UserAgent;
    use crate::shared::user_agent::constants::DEFAULT_SEC_USER_AGENT;

    fn sec_client_with_other_user_agent() -> SecClient {
        let user_agent = UserAgent::new("Other Company other@example.com")
            .expect("A hardcoded SEC-format string should always validate");
        let inner = ReqwestClient::new(user_agent)
            .expect("Given a validated user agent, building the HTTP client fails only if the TLS backend cannot initialize");
        SecClient::new(inner)
    }

    #[test]
    fn should_serialize_the_user_agent_when_serialized_to_json() {
        let client = SecClient::default();

        let expected_result = serde_json::json!({ "user_agent": DEFAULT_SEC_USER_AGENT });

        let result = serde_json::to_value(&client).expect("SecClient should serialize to JSON");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_default_user_agent_when_created_with_default() {
        let client = SecClient::default();

        let expected_result = &UserAgent::default();

        let result = client.user_agent();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_be_equal_when_user_agents_differ() {
        let client = SecClient::default();
        let other = sec_client_with_other_user_agent();

        let expected_result = false;

        let result = client == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_equal_when_user_agents_match() {
        let client = SecClient::default();
        let other = SecClient::default();

        let expected_result = true;

        let result = client == other;

        assert_eq!(result, expected_result);
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
