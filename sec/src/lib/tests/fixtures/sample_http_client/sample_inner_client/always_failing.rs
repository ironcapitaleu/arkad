use async_trait::async_trait;

use crate::shared::http_client::traits::InnerClient;

/// A fake HTTP client implementation that always returns a fixed error response.
/// This client is used for testing the error response handling in the [`HttpClient`] trait implementation by simulating a predefined error scenario.
#[derive(Debug, Clone)]
pub struct AlwaysFailingHttpClient;

#[async_trait]
impl InnerClient for AlwaysFailingHttpClient {
    type Request = ();
    type Response = String;
    type Error = String;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        Err(format!(
            "Simulated network error for request: {:?}",
            request
        ))
    }
}
