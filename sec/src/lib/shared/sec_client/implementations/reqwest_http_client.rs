//! # Reqwest HTTP Client Implementation
//!
//! This module provides a concrete implementation of the [`HttpClient`] trait using
//! the `reqwest` library. This is the default HTTP client implementation used by
//! [`super::super::SecClient`] for making HTTP requests to SEC endpoints.
//!
//! ## Types
//! - [`ReqwestHttpClient`]: Wrapper around `reqwest::Client` that implements [`HttpClient`].
//!
//! ## See Also
//! - [`super::super::traits::http_client::HttpClient`]: The trait this implementation fulfills.
//! - [`reqwest::Client`]: The underlying HTTP client.

use async_trait::async_trait;
use reqwest::Client;

use super::super::traits::http_client::HttpClient;
use crate::shared::sec_request::SecRequest;
use crate::shared::sec_request::sec_request_error::SecRequestError;
use crate::shared::sec_response::SecResponse;

/// Implementation of [`HttpClient`] using the `reqwest` library.
///
/// `ReqwestHttpClient` wraps a `reqwest::Client` and implements the [`HttpClient`]
/// trait, enabling it to be used with [`super::super::SecClient`] for dependency injection.
///
/// # Examples
///
/// ```rust
/// use reqwest::ClientBuilder;
/// use sec::shared::sec_client::ReqwestHttpClient;
///
/// // Create a reqwest client with custom configuration
/// let reqwest_client = ClientBuilder::new()
///     .user_agent("Sample Corp contact@sample.com")
///     .build()?;
///
/// // Wrap it in ReqwestHttpClient
/// let http_client = ReqwestHttpClient::new(reqwest_client);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct ReqwestHttpClient {
    inner: Client,
}

impl ReqwestHttpClient {
    /// Creates a new `ReqwestHttpClient` wrapping the provided `reqwest::Client`.
    ///
    /// # Arguments
    /// * `client` - The `reqwest::Client` to wrap.
    ///
    /// # Returns
    /// Returns a new `ReqwestHttpClient` instance.
    #[must_use]
    pub const fn new(client: Client) -> Self {
        Self { inner: client }
    }

    /// Returns a reference to the underlying `reqwest::Client`.
    #[must_use]
    pub const fn client(&self) -> &Client {
        &self.inner
    }
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn execute_request(&self, request: SecRequest) -> Result<SecResponse, SecRequestError> {
        let resp = self.inner.execute(request.inner).await;

        match resp {
            Err(e) => {
                let sec_error: SecRequestError = e.into();
                Err(sec_error)
            }
            Ok(resp) => match SecResponse::from_response(resp).await {
                Ok(sec_response) => Ok(sec_response),
                Err(e) => {
                    let sec_error: SecRequestError = e.into();
                    Err(sec_error)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_reqwest_http_client_when_valid_reqwest_client_is_provided() {
        let reqwest_client = Client::new();

        let result = ReqwestHttpClient::new(reqwest_client);

        assert!(result.client().get("https://example.com").build().is_ok());
    }

    #[test]
    fn should_return_inner_client_when_client_method_is_called() {
        let reqwest_client = Client::new();
        let http_client = ReqwestHttpClient::new(reqwest_client);

        let result = http_client.client();

        assert!(std::ptr::eq(result, &http_client.inner));
    }

    #[test]
    fn should_clone_reqwest_http_client_when_clone_is_called() {
        let reqwest_client = Client::new();
        let http_client = ReqwestHttpClient::new(reqwest_client);

        let result = http_client.clone();

        assert!(result.client().get("https://example.com").build().is_ok());
    }
}
