//! # SEC Client Trait
//!
//! This module defines the [`SecClient`] trait that represents the interface for SEC-compliant
//! HTTP clients. Implementations of this trait can use different underlying HTTP client
//! implementations via the [`super::http_client::HttpClient`] trait.

use async_trait::async_trait;

use crate::shared::sec_request::SecRequest;
use crate::shared::sec_request::implementations::reqwest_request::ReqwestRequest;
use crate::shared::sec_request::sec_request_error::SecRequestError;
use crate::shared::sec_response::SecResponse;

use super::http_client::HttpClient;

/// Trait defining the interface for SEC-compliant HTTP clients.
///
/// This trait provides the core functionality needed for making HTTP requests to SEC endpoints.
/// Implementations can use different underlying HTTP client types via the associated `Inner` type.
///
/// # Type Parameters
/// - `Inner`: The underlying HTTP client implementation, must implement [`HttpClient`].
#[async_trait]
pub trait SecClient: Send + Sync + Clone {
    /// The underlying HTTP client type.
    type Inner: HttpClient;

    /// Returns a reference to the underlying HTTP client implementation.
    fn inner(&self) -> &Self::Inner;

    /// Returns the unique identifier for this client instance.
    fn id(&self) -> &str;

    /// Executes the given `SecRequest` using the underlying HTTP client and returns a `SecResponse`.
    ///
    /// # Arguments
    /// * `request` - The `SecRequest` to be executed.
    ///
    /// # Returns
    /// Returns a `SecResponse` containing the response data from the executed request.
    ///
    /// # Errors
    /// This method will return a `SecRequestError` if:
    /// - The HTTP request fails (network issues, timeouts, etc.)
    /// - The response body cannot be read or parsed
    /// - Any other HTTP-related error occurs during execution
    async fn execute_request(
        &self,
        request: SecRequest<ReqwestRequest>,
    ) -> Result<SecResponse, SecRequestError> {
        self.inner().execute_request(request).await
    }
}
