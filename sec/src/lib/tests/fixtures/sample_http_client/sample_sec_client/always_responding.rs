use async_trait::async_trait;

use crate::shared::http_client::{InnerClient, SecClient};
use crate::shared::rate_limiter::RateLimiter;
use crate::tests::fixtures::sample_http_client::sample_inner_client::AlwaysRespondingHttpClient;
use crate::tests::fixtures::sample_rate_limiter::always_ready::AlwaysReadyRateLimiter;

/// A fake SEC client whose inner client always returns a `reqwest::Response` built from fixed
/// parts.
///
/// Used for testing code that takes any [`SecClient`] with a `reqwest` inner client, without a
/// network call or rate-limiter delay.
#[derive(Debug)]
pub struct FakeSecClient {
    /// The inner client that builds every response.
    pub inner: AlwaysRespondingHttpClient,
    /// The rate limiter, which never waits.
    pub rate_limiter: AlwaysReadyRateLimiter,
}

impl FakeSecClient {
    /// Creates a [`FakeSecClient`] whose responses have the given status, headers, and body.
    #[must_use]
    pub const fn new(status: u16, headers: Vec<(&'static str, String)>, body: Vec<u8>) -> Self {
        Self {
            inner: AlwaysRespondingHttpClient {
                status,
                headers,
                body,
            },
            rate_limiter: AlwaysReadyRateLimiter,
        }
    }
}

#[async_trait]
impl SecClient for FakeSecClient {
    type Inner = AlwaysRespondingHttpClient;
    type Limiter = AlwaysReadyRateLimiter;
    type Request = reqwest::Request;
    type Response = reqwest::Response;
    type Error = reqwest::Error;

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
        self.rate_limiter().await_turn().await;
        self.inner().execute_request(request).await
    }
}
