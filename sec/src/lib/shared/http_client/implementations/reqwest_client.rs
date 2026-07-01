//! # Reqwest Inner Client
//!
//! Implements [`InnerClient`] for [`reqwest::Client`], binding the transport's request, response,
//! and error types and delegating execution to `reqwest`.

use async_trait::async_trait;
use reqwest::{Client, Error as ReqwestError, Request, Response};

use super::super::traits::InnerClient;

#[async_trait]
impl InnerClient for Client {
    /// The [`reqwest::Request`] type.
    type Request = Request;
    /// The [`reqwest::Response`] type.
    type Response = Response;
    /// The [`reqwest::Error`] type.
    type Error = ReqwestError;

    async fn execute_request(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        self.execute(request).await
    }
}
