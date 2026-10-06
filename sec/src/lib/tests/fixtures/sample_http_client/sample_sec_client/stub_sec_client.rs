use async_trait::async_trait;
use serde::{Serialize, Serializer};

use crate::shared::http_client::SecClient;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::rate_limiter::RateLimiter;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::tests::fixtures::sample_http_client::sample_inner_client::AlwaysSucceedingHttpClient;
use crate::tests::fixtures::sample_rate_limiter::always_ready::AlwaysReadyRateLimiter;

/// Fake SEC client that returns a preset result for every request, without network access.
///
/// It uses the same request, response, and error types as the real `SecClient`, so it can replace
/// that client in any state that is generic over the SEC client.
// Deviation: the testing skill names domain-level fakes `Fake{ConceptName}`. `FakeSecClient` already
// names the placeholder fake in `always_succeeding.rs`, and this type returns a preset result, so
// it is named `StubSecClient`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StubSecClient {
    result: Result<SecResponse, FailedSecRequest>,
}

impl StubSecClient {
    /// Creates a stub that returns `response` for every request.
    #[must_use]
    pub const fn succeeding(response: SecResponse) -> Self {
        Self {
            result: Ok(response),
        }
    }

    /// Creates a stub that returns `error` for every request.
    #[must_use]
    pub const fn failing(error: FailedSecRequest) -> Self {
        Self { result: Err(error) }
    }
}

impl Serialize for StubSecClient {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit_struct("StubSecClient")
    }
}

#[async_trait]
impl SecClient for StubSecClient {
    type Inner = AlwaysSucceedingHttpClient;
    type Limiter = AlwaysReadyRateLimiter;
    type Request = SecRequest;
    type Response = SecResponse;
    type Error = FailedSecRequest;

    fn inner(&self) -> &Self::Inner {
        &AlwaysSucceedingHttpClient
    }

    fn rate_limiter(&self) -> &Self::Limiter {
        &AlwaysReadyRateLimiter
    }

    async fn execute_sec_request(
        &self,
        _request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        self.rate_limiter().await_turn().await;
        self.result.clone()
    }
}
