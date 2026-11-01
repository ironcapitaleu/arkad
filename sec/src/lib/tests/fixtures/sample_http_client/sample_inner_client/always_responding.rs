use async_trait::async_trait;
use reqwest::ResponseBuilderExt;

use crate::shared::http_client::traits::InnerClient;

/// A fake HTTP client that always returns a `reqwest::Response` built from fixed parts.
///
/// Used for testing code that reads a real `reqwest::Response` without a network call. Each call
/// builds a new response, because `reqwest::Response` is not `Clone`. The response URL is the
/// request URL.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AlwaysRespondingHttpClient {
    /// The HTTP status code of every response.
    pub status: u16,
    /// The header names and values of every response.
    pub headers: Vec<(&'static str, String)>,
    /// The body of every response.
    pub body: Vec<u8>,
}

#[async_trait]
impl InnerClient for AlwaysRespondingHttpClient {
    type Request = reqwest::Request;
    type Response = reqwest::Response;
    type Error = reqwest::Error;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let mut builder = http::Response::builder()
            .status(self.status)
            .url(request.url().clone());
        for (name, value) in &self.headers {
            builder = builder.header(*name, value);
        }
        let response = builder
            .body(self.body.clone())
            .expect("Given a valid status code and header names, building the response should always succeed");
        Ok(reqwest::Response::from(response))
    }
}
