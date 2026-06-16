//! # SEC Client
//!
//! Provides the [`SecClient`], the concrete [`SecClient`](crate::shared::http_client::SecClient)
//! used throughout the pipeline.
//!
//! ## Modules
//!
//! - [`error`]: The [`FailedSecRequest`] error returned when execution fails.

use async_trait::async_trait;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::shared::http_client::InnerClient;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::shared::user_agent::UserAgent;
use crate::shared::user_agent::constants::DEFAULT_SEC_USER_AGENT;

use self::error::FailedSecRequest;

pub mod error;

/// The default SEC API client, driving the full `SecRequest` → `SecResponse` cycle.
///
/// Executes a validated request through a `reqwest::Client` and validates the reply into a
/// [`SecResponse`]. Construct one with [`SecClient::default`] for the standard SEC-compliant setup,
/// or [`SecClient::new`] to supply your own transport.
///
/// # Cloning
///
/// `reqwest::Client` is `Arc`-backed, so clones share one connection pool, TLS sessions, and DNS
/// cache — no extra `Arc` wrapping is needed for concurrent use.
///
/// # User Agent
///
/// [`SecClient::default`] sets the `User-Agent` from a validated [`UserAgent`], guaranteeing every
/// request carries the SEC-compliant header the API requires.
#[derive(Debug, Clone)]
pub struct SecClient {
    inner: reqwest::Client,
}

impl Serialize for SecClient {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let state = serializer.serialize_struct("SecClient", 0)?;
        state.end()
    }
}

impl SecClient {
    /// Creates a new `SecClient` with the given `reqwest::Client`.
    #[must_use]
    pub const fn new(inner: reqwest::Client) -> Self {
        Self { inner }
    }
}

/// Creates a default `SecClient` configured with the default SEC user agent.
impl Default for SecClient {
    fn default() -> Self {
        let user_agent = UserAgent::new(DEFAULT_SEC_USER_AGENT)
            .expect("The default SEC user agent constant should always be valid");
        let http_client = reqwest::Client::builder()
            .user_agent(user_agent.inner())
            .build()
            .expect("A validated UserAgent should always produce a valid HTTP client");
        Self::new(http_client)
    }
}

// Deviation: `reqwest::Client` does not expose any comparable or hashable state,
// so all `SecClient` instances are considered equal. This satisfies trait bounds
// that require `Eq + Ord + Hash` (e.g. for use in collections or state machines).

impl PartialEq for SecClient {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for SecClient {}

impl std::hash::Hash for SecClient {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}

impl PartialOrd for SecClient {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SecClient {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

#[async_trait]
impl SecClientTrait for SecClient {
    type Inner = reqwest::Client;
    type Request = SecRequest;
    type Response = SecResponse;
    type Error = FailedSecRequest;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    async fn execute_sec_request(
        &self,
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        let inner_request = request.into_inner();
        let inner_response = self.inner.execute_request(inner_request).await?;
        let sec_response = SecResponse::from_inner(inner_response).await?;
        Ok(sec_response)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::SecClient;

    #[test]
    fn should_serialize_to_empty_struct_when_serialized_to_json() {
        let client = SecClient::default();

        let expected_result = serde_json::json!({});

        let result = serde_json::to_value(&client).expect("SecClient should serialize to JSON");

        assert_eq!(result, expected_result);
    }

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_unpin<T: Unpin>() {}

    #[test]
    fn should_be_send() {
        assert_send::<SecClient>();
    }

    #[test]
    fn should_be_sync() {
        assert_sync::<SecClient>();
    }

    #[test]
    fn should_be_unpin() {
        assert_unpin::<SecClient>();
    }
}
