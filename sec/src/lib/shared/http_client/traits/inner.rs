use std::fmt::Debug;

use async_trait::async_trait;

/// The transport-level HTTP client: executes a raw request and returns a raw response.
///
/// Abstracts over any concrete (often third-party) HTTP client so the library can swap transports
/// without touching the domain layer. Implemented for [`reqwest::Client`] and the test fakes;
/// [`SecClient`](super::SecClient) builds on top of it.
///
/// # Associated Types
///
/// Each implementor binds these to its transport's concrete types, which is what lets the domain
/// layer stay independent of any specific HTTP crate:
///
/// - `Request`: The type representing the transport's request.
/// - `Response`: The type representing the transport's response, returned on success.
/// - `Error`: The error type returned on failure.
#[async_trait]
pub trait InnerClient: Send + Sync + Debug + Clone {
    /// The type representing the transport's request.
    type Request;
    /// The type representing the transport's response, returned on success.
    type Response;
    /// The error type returned on failure.
    type Error;

    /// Executes an HTTP request, returning the transport's response.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the request fails at the transport level.
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
