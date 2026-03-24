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
    #[must_use] 
    pub const fn new(client: Client) -> Self {
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
    /// This is the [`reqwest::Request`] type from the [reqwest] library.
    type Request = Request;
    /// This is the [`reqwest::Response`] type from the [reqwest] library.
    type Response = Response;
    /// This is the [`reqwest::Error`] type from the [reqwest] library.
    type Error = ReqwestError;

    /// Executes a given HTTP request asynchronously.
    /// Takes a [Request] as input.
    /// Returns a [Response] on success or an [`ReqwestError`] on failure.
    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        
        self.client.execute(request).await
    }
}
