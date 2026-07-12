use std::fmt::Debug;

use async_trait::async_trait;

use crate::shared::http_client::InnerClient;
use crate::shared::rate_limiter::RateLimiter;

/// The domain-level SEC HTTP client: executes an SEC request and returns an SEC response.
///
/// Sits above [`InnerClient`], adding the SEC-specific knowledge (endpoints, request/response
/// shaping) that the raw HTTP client lacks.
///
/// # Associated Types
///
/// Each implementor chooses the concrete types filling these slots, which is what keeps the trait
/// decoupled from any specific HTTP crate:
///
/// - `Inner`: The raw HTTP client this trait delegates to. Must implement [`InnerClient`].
/// - `Limiter`: The rate limiter pacing outgoing requests. Must implement [`RateLimiter`].
/// - `Request`: The SEC request type accepted by [`SecClient::execute_sec_request`].
/// - `Response`: The SEC response type returned on success.
/// - `Error`: The error type returned when request execution or response validation fails.
#[async_trait]
pub trait SecClient: Send + Sync + Debug {
    /// The raw HTTP client this trait delegates to. Must implement [`InnerClient`].
    type Inner: InnerClient;
    /// The rate limiter pacing outgoing requests. Must implement [`RateLimiter`].
    type Limiter: RateLimiter;
    /// The SEC response type returned on success.
    type Response;
    /// The error type returned when request execution or response validation fails.
    type Error;
    /// The SEC request type this client executes.
    type Request;

    /// Returns a reference to the underlying raw HTTP client.
    fn inner(&self) -> &Self::Inner;

    /// Returns a reference to the rate limiter pacing outgoing requests.
    fn rate_limiter(&self) -> &Self::Limiter;

    /// Executes an SEC request, returning the validated response.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the request fails at the network level or the response fails
    /// SEC validation.
    async fn execute_sec_request(
        &self,
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error>;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::shared::http_client::traits::SecClient;
    use crate::tests::fixtures::sample_http_client::sample_inner_client::AlwaysSucceedingHttpClient;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_succeeding::FakeSecClient;

    #[tokio::test]
    async fn should_return_expected_success_response_for_fake_sec_client() {
        let client = FakeSecClient::new();
        let request = ();
        let expected_success_message =
            String::from("Simulated success response for sec request: ()");

        let expected_result = Ok(expected_success_message);

        let result = client.execute_sec_request(request).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_expected_inner_client_for_fake_sec_client() {
        let fake_client = FakeSecClient::new();
        let expected_inner_client = AlwaysSucceedingHttpClient;

        let expected_result = &expected_inner_client;

        let result = fake_client.inner();

        assert_eq!(result, expected_result);
    }
}
