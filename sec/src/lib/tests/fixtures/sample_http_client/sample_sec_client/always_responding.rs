use std::hash::Hash;

use serde::Serialize;
use url::Url;

use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::http_client::{RetryPolicy, SecClient};
use crate::shared::rate_limiter::RateLimiter;
use crate::shared::response::implementations::sec_response::{ResponseBody, SecResponse};
use crate::shared::user_agent::UserAgent;
use crate::tests::fixtures::sample_http_client::sample_inner_client::routing::RequestCounts;
use crate::tests::fixtures::sample_http_client::sample_inner_client::{
    AlwaysRespondingHttpClient, Delivery,
};
use crate::tests::fixtures::sample_rate_limiter::always_ready::AlwaysReadyRateLimiter;

/// A fake SEC client whose inner client always returns a `reqwest::Response` built from fixed
/// parts.
///
/// Used for testing code that takes any [`SecClient`], without a network call. The rate limiter
/// `L` never waits. [`AlwaysRespondingSecClient::with_rate_limiter`] swaps it, for example for a
/// limiter that counts its turns. The client sends once, unless
/// [`AlwaysRespondingSecClient::with_retry_policy`] sets a policy. It then retries a transient
/// failure and counts each retry by URL and retry number in [`AlwaysRespondingSecClient::retries`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct AlwaysRespondingSecClient<L = AlwaysReadyRateLimiter> {
    /// The inner client that builds every response.
    #[serde(skip)]
    pub inner: AlwaysRespondingHttpClient,
    /// The rate limiter.
    #[serde(skip)]
    pub rate_limiter: L,
    /// The `User-Agent` the client reports.
    pub user_agent: UserAgent,
    /// The attempts and the waits of one request.
    #[serde(skip)]
    pub retry_policy: RetryPolicy,
    /// The number of retries per key `<URL> retry <number>`, shared by all clones.
    #[serde(skip)]
    pub retries: RequestCounts,
}

impl AlwaysRespondingSecClient {
    /// Creates an [`AlwaysRespondingSecClient`] whose responses have the given status, headers, and body.
    ///
    /// The client reports the `User-Agent` `Fake Notes Client fake@example.com`.
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded `User-Agent` fails validation.
    #[must_use]
    pub fn new(status: u16, headers: Vec<(&'static str, String)>, body: Vec<u8>) -> Self {
        Self {
            inner: AlwaysRespondingHttpClient {
                status,
                headers,
                body,
                delivery: Delivery::Full,
            },
            rate_limiter: AlwaysReadyRateLimiter,
            user_agent: UserAgent::new("Fake Notes Client fake@example.com").expect(
                "Given a hardcoded SEC-format user agent, validation should always succeed",
            ),
            retry_policy: RetryPolicy::SEND_ONCE,
            retries: RequestCounts::default(),
        }
    }
}

impl<L> AlwaysRespondingSecClient<L> {
    /// Returns the client with `rate_limiter` in place of its rate limiter.
    #[must_use]
    pub fn with_rate_limiter<M>(self, rate_limiter: M) -> AlwaysRespondingSecClient<M> {
        AlwaysRespondingSecClient {
            inner: self.inner,
            rate_limiter,
            user_agent: self.user_agent,
            retry_policy: self.retry_policy,
            retries: self.retries,
        }
    }

    /// Returns the client set to retry a transient failure under `retry_policy`.
    #[must_use]
    pub const fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }
}

impl<L> SecClient for AlwaysRespondingSecClient<L>
where
    L: RateLimiter + Clone + Eq + Ord + Hash + Unpin,
{
    type Inner = AlwaysRespondingHttpClient;
    type Limiter = L;
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

    fn retry_policy(&self) -> RetryPolicy {
        self.retry_policy
    }

    fn is_transient(&self, error: &FailedSecRequest) -> bool {
        error.is_transient()
    }

    fn on_retry(&self, _error: &FailedSecRequest, url: &Url, retry: u32) {
        let _ = self.retries.increment(&format!("{url} retry {retry}"));
    }
}
