use std::fmt::Debug;

use async_trait::async_trait;

/// A trait defining the methods an inner http client is expected to implement. This is used to enforce a consistent interface for any (third party) HTTP client that might be used.
#[async_trait]
pub trait InnerClient: Send + Sync + Debug + Clone {
    type Request;
    type Response;
    type Error;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error>;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::shared::http_client::traits::InnerClient;
    use crate::tests::common::sample_http_client::sample_inner_client::always_failing::AlwaysFailingHttpClient;

    #[tokio::test]
    async fn should_return_error_for_always_failing_http_client() {
        let client = AlwaysFailingHttpClient;
        let expected_error_message = String::from("Simulated network error for request: ()");

        let expected_result = Err(expected_error_message);

        let result = client.execute_request(()).await;

        assert_eq!(result, expected_result);
    }
}
