use serde::Serialize;

use crate::shared::http_client::SecClient;
use crate::shared::response::implementations::sec_response::ResponseBody;
use crate::shared::user_agent::UserAgent;
use crate::tests::fixtures::sample_http_client::sample_inner_client::AlwaysReturningHttpClient;
use crate::tests::fixtures::sample_rate_limiter::always_ready::AlwaysReadyRateLimiter;
use crate::tests::fixtures::sample_response::sample_inner_response::FakeInnerResponse;
use crate::tests::fixtures::sample_response::sample_sec_response::FakeSecResponse;

/// Fake `SecClient` that returns a [`FakeSecResponse`] with the body `{"key": "value"}`, or the
/// error of its inner client.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct FakeSecClient {
    #[serde(skip)]
    pub inner: AlwaysReturningHttpClient,
    #[serde(skip)]
    pub rate_limiter: AlwaysReadyRateLimiter,
    pub user_agent: UserAgent,
}

impl Default for FakeSecClient {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeSecClient {
    /// Creates a [`FakeSecClient`] with the default [`UserAgent`] whose inner client returns
    /// [`fake_json_inner_response`].
    #[must_use]
    pub fn new() -> Self {
        Self::with_inner_result(Ok(fake_json_inner_response()))
    }

    /// Creates a [`FakeSecClient`] whose inner client fails with `error`.
    #[must_use]
    pub fn failing(error: &str) -> Self {
        Self::with_inner_result(Err(error.to_string()))
    }

    fn with_inner_result(result: Result<FakeInnerResponse, String>) -> Self {
        Self {
            inner: AlwaysReturningHttpClient { result },
            rate_limiter: AlwaysReadyRateLimiter,
            user_agent: UserAgent::default(),
        }
    }
}

/// Returns the inner response of [`FakeSecClient::new`]: status 200 and the JSON body
/// `{"key": "value"}`.
#[must_use]
pub fn fake_json_inner_response() -> FakeInnerResponse {
    FakeInnerResponse::new(
        String::from("https://example.com/"),
        String::from("{\"key\": \"value\"}"),
        String::new(),
        200,
        String::from("application/json"),
    )
}

impl SecClient for FakeSecClient {
    type Inner = AlwaysReturningHttpClient;
    type Limiter = AlwaysReadyRateLimiter;
    type Response<B: ResponseBody> = FakeSecResponse<B>;
    type Error = String;

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
