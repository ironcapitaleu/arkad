use async_trait::async_trait;
use reqwest::{Client, Error as ReqwestError, Request, Response};

use super::super::traits::InnerClient;

#[async_trait]
impl InnerClient for Client {
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
        self.execute(request).await
    }
}
