use serde::Serialize;

use crate::shared::http_client::SecClient;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::response::implementations::sec_response::{ResponseBody, SecResponse};
use crate::shared::user_agent::UserAgent;
use crate::tests::fixtures::sample_http_client::sample_inner_client::{
    AlwaysRespondingHttpClient, RoutingHttpClient,
};
use crate::tests::fixtures::sample_rate_limiter::always_ready::AlwaysReadyRateLimiter;

/// A fake SEC client that answers each URL with its own fixed response.
///
/// A URL without a route gets an empty 404 response. The rate limiter never waits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct RoutingSecClient {
    /// The inner client that routes every request.
    #[serde(skip)]
    pub inner: RoutingHttpClient,
    /// The rate limiter.
    #[serde(skip)]
    pub rate_limiter: AlwaysReadyRateLimiter,
    /// The `User-Agent` the client reports.
    pub user_agent: UserAgent,
}

impl Default for RoutingSecClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutingSecClient {
    /// Creates a [`RoutingSecClient`] with no routes.
    ///
    /// # Panics
    ///
    /// Never panics: the hardcoded `User-Agent` always passes validation.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: RoutingHttpClient { routes: Vec::new() },
            rate_limiter: AlwaysReadyRateLimiter,
            user_agent: UserAgent::new("Fake Routing Client fake@example.com").expect(
                "Given a hardcoded SEC-format user agent, validation should always succeed",
            ),
        }
    }

    /// Adds a route that answers `url` with `status`, `content_type`, and `body`.
    #[must_use]
    pub fn route(mut self, url: &str, status: u16, content_type: &str, body: &[u8]) -> Self {
        let responder = AlwaysRespondingHttpClient {
            status,
            headers: vec![("content-type", content_type.to_string())],
            body: body.to_vec(),
        };
        self.inner.routes.push((url.to_string(), responder));
        self
    }
}

impl SecClient for RoutingSecClient {
    type Inner = RoutingHttpClient;
    type Limiter = AlwaysReadyRateLimiter;
    type Response<B: ResponseBody> = SecResponse<B>;
    type Error = FailedSecRequest;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn rate_limiter(&self) -> &Self::Limiter {
        &self.rate_limiter
    }

    fn user_agent(&self) -> &UserAgent {
        &self.user_agent
    }
}
