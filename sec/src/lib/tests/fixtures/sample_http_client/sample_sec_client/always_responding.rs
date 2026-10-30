use std::hash::Hash;

use serde::Serialize;

use crate::shared::http_client::SecClient;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::rate_limiter::RateLimiter;
use crate::shared::response::implementations::sec_response::{ResponseBody, SecResponse};
use crate::shared::user_agent::UserAgent;
use crate::tests::fixtures::sample_http_client::sample_inner_client::AlwaysRespondingHttpClient;
use crate::tests::fixtures::sample_rate_limiter::always_ready::AlwaysReadyRateLimiter;

/// A fake SEC client whose inner client always returns a `reqwest::Response` built from fixed
/// parts.
///
/// Used for testing code that takes any [`SecClient`], without a network call. The rate limiter
/// `L` never waits. [`AlwaysRespondingSecClient::with_rate_limiter`] swaps it, for example for a
/// limiter that counts its turns.
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
            },
            rate_limiter: AlwaysReadyRateLimiter,
            user_agent: UserAgent::new("Fake Notes Client fake@example.com").expect(
                "Given a hardcoded SEC-format user agent, validation should always succeed",
            ),
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
        }
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
}
