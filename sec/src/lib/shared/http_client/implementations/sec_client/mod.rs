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
use crate::shared::response::implementations::sec_notes_file_response::SecNotesFileResponse;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::shared::user_agent::UserAgent;

use self::error::FailedSecRequest;
use super::reqwest_client::ReqwestClient;

pub mod error;

/// Shared rate limiter backing all [`SecClient`] instances.
static GLOBAL_RATE_LIMITER: OnceLock<SecRateLimiter> = OnceLock::new();

/// The default SEC API client, driving the full `SecRequest` → `SecResponse` cycle.
///
/// Executes a validated request through a [`ReqwestClient`] and validates the reply into a
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

    /// Downloads one Notes file and returns it as a [`SecNotesFileResponse`].
    ///
    /// Waits on the rate limiter, sends the request, and reads the whole body.
    ///
    /// # Errors
    ///
    /// Returns a [`FailedSecRequest`] if the request fails at the network level, or if the
    /// response fails a check of [`SecNotesFileResponse::from_parts`].
    pub async fn download_sec_file(
        &self,
        request: SecRequest,
    ) -> Result<SecNotesFileResponse, FailedSecRequest> {
        download_notes_file(&self.inner, &self.rate_limiter, request.into_inner()).await
    }
}

/// Waits on `rate_limiter`, sends `request` through `client`, and validates the reply.
///
/// Generic over the client, so tests can send the request through a fake [`InnerClient`].
///
/// # Errors
///
/// Returns a [`FailedSecRequest`] if the request fails at the network level, or if the
/// response fails a check of [`SecNotesFileResponse::from_parts`].
async fn download_notes_file<C>(
    client: &C,
    rate_limiter: &impl RateLimiter,
    request: reqwest::Request,
) -> Result<SecNotesFileResponse, FailedSecRequest>
where
    C: InnerClient<
            Request = reqwest::Request,
            Response = reqwest::Response,
            Error = reqwest::Error,
        >,
{
    rate_limiter.await_turn().await;

    let inner_response = client.execute_request(request).await?;
    let notes_file_response = SecNotesFileResponse::from_inner(inner_response).await?;
    Ok(notes_file_response)
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

#[async_trait]
impl SecClientTrait for SecClient {
    type Inner = ReqwestClient;
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
    use pretty_assertions::assert_eq;

    use super::{SecClient, download_notes_file};
    use crate::shared::content_type::ContentType;
    use crate::shared::http_client::implementations::reqwest_client::ReqwestClient;
    use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
    use crate::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use crate::shared::status_code::StatusCode;
    use crate::shared::user_agent::UserAgent;
    use crate::shared::user_agent::constants::DEFAULT_SEC_USER_AGENT;
    use crate::tests::fixtures::sample_http_client::sample_inner_client::AlwaysRespondingHttpClient;
    use crate::tests::fixtures::sample_rate_limiter::always_ready::AlwaysReadyRateLimiter;

    const NOTES_FILE_BODY: &[u8] = b"PK\x03\x04 notes";

    fn notes_file_request() -> reqwest::Request {
        let url = "https://www.sec.gov/files/dera/data/financial-statement-and-notes-data-sets/2024q1_notes.zip";
        let url = reqwest::Url::parse(url)
            .expect("Given a hardcoded valid URL, parsing should always succeed");
        reqwest::Request::new(reqwest::Method::GET, url)
    }

    fn responding_client(status: u16, content_type: &str) -> AlwaysRespondingHttpClient {
        AlwaysRespondingHttpClient {
            status,
            headers: vec![
                ("content-type", content_type.to_string()),
                ("content-length", NOTES_FILE_BODY.len().to_string()),
            ],
            body: NOTES_FILE_BODY.to_vec(),
        }
    }

    fn invalid_response(reason: ErrorReason) -> FailedSecRequest {
        FailedSecRequest::from(InvalidSecResponse::new(reason))
    }

    #[tokio::test]
    async fn should_return_the_body_when_the_notes_file_download_succeeds() {
        let client = responding_client(200, "application/octet-stream");

        let expected_result = NOTES_FILE_BODY.to_vec();

        let result = download_notes_file(&client, &AlwaysReadyRateLimiter, notes_file_request())
            .await
            .expect("Given status 200, an octet stream and a matching length, the download should always succeed")
            .body()
            .to_vec();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_status_code_when_the_notes_file_is_not_found() {
        let client = responding_client(404, "application/octet-stream");

        let expected_result = Some(invalid_response(ErrorReason::InvalidStatusCode {
            status_code: StatusCode::NotFound,
        }));

        let result = download_notes_file(&client, &AlwaysReadyRateLimiter, notes_file_request())
            .await
            .err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_content_type_when_the_notes_file_is_html() {
        let client = responding_client(200, "text/html");

        let expected_result = Some(invalid_response(ErrorReason::InvalidContentType {
            content_type: ContentType::Html,
        }));

        let result = download_notes_file(&client, &AlwaysReadyRateLimiter, notes_file_request())
            .await
            .err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_content_length_when_the_notes_file_body_is_short() {
        let client = AlwaysRespondingHttpClient {
            body: NOTES_FILE_BODY[..4].to_vec(),
            ..responding_client(200, "application/zip")
        };

        let expected_result = Some(invalid_response(ErrorReason::InvalidContentLength {
            content_length: NOTES_FILE_BODY.len().to_string(),
            body_length: 4,
        }));

        let result = download_notes_file(&client, &AlwaysReadyRateLimiter, notes_file_request())
            .await
            .err();

        assert_eq!(result, expected_result);
    }

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
