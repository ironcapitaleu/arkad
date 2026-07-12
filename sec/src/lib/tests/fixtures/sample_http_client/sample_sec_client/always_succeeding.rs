use async_trait::async_trait;

use crate::shared::http_client::SecClient;
use crate::shared::rate_limiter::RateLimiter;
use crate::tests::fixtures::sample_http_client::sample_inner_client::AlwaysSucceedingHttpClient;
use crate::tests::fixtures::sample_rate_limiter::always_ready::AlwaysReadyRateLimiter;

/// Fake SecClient that always returns a successful fixed string response.
#[derive(Debug)]
pub struct FakeSecClient {
    pub inner: AlwaysSucceedingHttpClient,
    pub rate_limiter: AlwaysReadyRateLimiter,
}

impl FakeSecClient {
    pub fn new() -> Self {
        Self {
            inner: AlwaysSucceedingHttpClient,
            rate_limiter: AlwaysReadyRateLimiter,
        }
    }
}

#[async_trait]
impl SecClient for FakeSecClient {
    type Inner = AlwaysSucceedingHttpClient;
    type Limiter = AlwaysReadyRateLimiter;
    type Request = ();
    type Response = String;
    type Error = String;

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
        Ok(format!(
            "Simulated success response for sec request: {:?}",
            request
        ))
    }
}
