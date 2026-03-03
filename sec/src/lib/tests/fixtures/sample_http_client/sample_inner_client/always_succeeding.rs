use async_trait::async_trait;

use crate::shared::http_client::traits::InnerClient;

/// A fake HTTP client implementation that always returns a fixed success response.
/// This client is used for testing the success response handling in the [`HttpClient`] trait implementation by simulating a predefined success scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct AlwaysSucceedingHttpClient;

#[async_trait]
impl InnerClient for AlwaysSucceedingHttpClient {
    type Request = ();
    type Response = String;
    type Error = String;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        Ok(format!(
            "Simulated success response for request: {:?}",
            request
        ))
    }
}
