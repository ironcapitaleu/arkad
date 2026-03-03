use std::fmt::Debug;

use async_trait::async_trait;

use crate::shared::http_client::InnerClient;

/// A trait defining the behavior a high-level SecClient is expected to implement. This is used to define operations on a level where domain-specific knowledge can be used to abstract away the inner handling of HTTP requests, endpoints, etc.
#[async_trait]
pub trait SecClient: Send + Sync + Debug {
    type Inner: InnerClient;
    type Response;
    type Error;
    type Request;

    fn inner(&self) -> &Self::Inner;
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
