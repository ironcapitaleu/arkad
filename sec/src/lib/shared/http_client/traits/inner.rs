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
    use crate::tests::fixtures::sample_http_client::sample_inner_client::{
        always_failing::AlwaysFailingHttpClient, always_succeeding::AlwaysSucceedingHttpClient,
    };

    #[tokio::test]
    async fn should_return_expected_error_for_always_failing_http_client() {
        let client = AlwaysFailingHttpClient;
        let request = ();
        let expected_error_message = String::from("Simulated network error for request: ()");

        let expected_result = Err(expected_error_message);

        let result = client.execute_request(request).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_expected_response_for_always_succeeding_http_client() {
        let client = AlwaysSucceedingHttpClient;
        let request = ();
        let expected_success_message = String::from("Simulated success response for request: ()");

        let expected_result = Ok(expected_success_message);

        let result = client.execute_request(request).await;

        assert_eq!(result, expected_result);
    }
}
