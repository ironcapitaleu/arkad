//! # Reqwest Inner Client
//!
//! Implements [`InnerClient`] for [`reqwest::Client`], binding the request, response, and error
//! types to their `reqwest` equivalents.

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
