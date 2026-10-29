use std::sync::atomic::AtomicUsize;

use async_trait::async_trait;

use crate::shared::http_client::{InnerClient, SecClient};
use crate::shared::rate_limiter::RateLimiter;
use crate::tests::fixtures::sample_http_client::sample_inner_client::AlwaysRespondingHttpClient;
use crate::tests::fixtures::sample_rate_limiter::always_ready::AlwaysReadyRateLimiter;
use crate::tests::fixtures::sample_rate_limiter::counting::CountingRateLimiter;

/// A fake SEC client whose inner client always returns a `reqwest::Response` built from fixed
/// parts.
///
/// Used for testing code that takes any [`SecClient`] with a `reqwest` inner client, without a
/// network call or rate-limiter delay. The rate limiter counts its turns.
#[derive(Debug)]
pub struct AlwaysRespondingSecClient {
    /// The inner client that builds every response.
    pub inner: AlwaysRespondingHttpClient,
    /// The rate limiter, which never waits and counts its turns.
    pub rate_limiter: CountingRateLimiter,
}

impl AlwaysRespondingSecClient {
    /// Creates an [`AlwaysRespondingSecClient`] whose responses have the given status, headers, and body.
    #[must_use]
    pub const fn new(status: u16, headers: Vec<(&'static str, String)>, body: Vec<u8>) -> Self {
        Self {
            inner: AlwaysRespondingHttpClient {
                status,
                headers,
                body,
            },
            rate_limiter: CountingRateLimiter {
                turns: AtomicUsize::new(0),
            },
        }
    }
}

#[async_trait]
impl SecClient for AlwaysRespondingSecClient {
    type Inner = AlwaysRespondingHttpClient;
    type Limiter = CountingRateLimiter;
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

/// A fake SEC client whose inner client always returns a `reqwest::Response` built from fixed
/// parts, and whose rate limiter never waits.
///
/// Unlike [`AlwaysRespondingSecClient`], it is a value type: `Clone`, `Eq`, `Ord`, and `Hash`. It
/// fits into a state context.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlwaysRespondingReadySecClient {
    /// The inner client that builds every response.
    pub inner: AlwaysRespondingHttpClient,
    /// The rate limiter, which never waits.
    pub rate_limiter: AlwaysReadyRateLimiter,
}

impl AlwaysRespondingReadySecClient {
    /// Creates an [`AlwaysRespondingReadySecClient`] whose responses have the given status, headers, and body.
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
impl SecClient for AlwaysRespondingReadySecClient {
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
