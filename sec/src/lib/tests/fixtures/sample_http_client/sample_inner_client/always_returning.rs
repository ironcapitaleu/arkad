use async_trait::async_trait;

use crate::shared::http_client::traits::InnerClient;
use crate::tests::fixtures::sample_response::sample_inner_response::FakeInnerResponse;

/// A fake HTTP client that takes a `reqwest::Request` and always returns a fixed
/// [`FakeInnerResponse`].
///
/// Used for testing a [`SecClient`](crate::shared::http_client::SecClient) that returns a fake
/// response, without a network call.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlwaysReturningHttpClient {
    /// The response of every request.
    pub response: FakeInnerResponse,
}

#[async_trait]
impl InnerClient for AlwaysReturningHttpClient {
    type Request = reqwest::Request;
    type Response = FakeInnerResponse;
    type Error = String;

    async fn execute_request(
        &self,
        _request: Self::Request,
    ) -> Result<Self::Response, Self::Error> {
        Ok(self.response.clone())
    }
}
