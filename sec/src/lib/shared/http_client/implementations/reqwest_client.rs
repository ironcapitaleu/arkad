use async_trait::async_trait;
use reqwest::{Client, Error as ReqwestError, Request, Response};

use super::super::traits::InnerClient;

/// An HTTP client implementation that wraps around the `reqwest` library's `reqwest::Client`.
/// This client can be used to execute arbitrary HTTP requests.
#[derive(Debug, Clone)]
pub struct ReqwestClient {
    client: Client,
}

impl ReqwestClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl InnerClient for ReqwestClient {
    /// This is the [reqwest::Request] type from the [reqwest] library.
    type Request = Request;
    /// This is the [reqwest::Response] type from the [reqwest] library.
    type Response = Response;
    /// This is the [reqwest::Error] type from the [reqwest] library.
    type Error = ReqwestError;

    /// Executes a given HTTP request asynchronously.
    /// Takes a [Request] as input.
    /// Returns a [Response] on success or an [ReqwestError] on failure.
    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let resp = self.client.execute(request).await;
        resp
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    #[ignore = "Reason: This test requires an internet connection."]
    async fn should_return_successful_response_for_valid_request() {
        let client = ReqwestClient::default();
        let url = "https://httpbin.org/get";
        let request = Request::new(reqwest::Method::GET, reqwest::Url::parse(url).unwrap());

        let expected_result = reqwest::StatusCode::OK;

        let result = client
            .execute_request(request)
            .await
            .expect(&format!("The URL `{url}` should always succeed"))
            .status();

        assert_eq!(result, expected_result);
    }
}
