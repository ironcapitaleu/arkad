//! # HTTP Client Trait
//!
//! This module defines the [`HttpClient`] trait which abstracts HTTP client functionality
//! for dependency injection.
//!
//! ## Types
//! - [`HttpClient`]: Trait defining the interface for HTTP client operations.
//!
//! ## See Also
//! - [`super::SecClient`]: Uses this trait for HTTP client abstraction.
//! - [`super::reqwest_http_client`]: Concrete implementation using reqwest.

use async_trait::async_trait;

use crate::shared::sec_request::SecRequest;
use crate::shared::sec_request::sec_request_error::SecRequestError;
use crate::shared::sec_response::SecResponse;

/// Trait defining the interface for HTTP client operations.
///
/// This trait abstracts HTTP client functionality to enable dependency injection
/// and testing. Any HTTP client implementation can be used with [`SecClient`] as
/// long as it implements this trait.
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Executes an HTTP request and returns the response.
    ///
    /// # Arguments
    /// * `request` - The [`SecRequest`] to execute.
    ///
    /// # Returns
    /// Returns a [`SecResponse`] containing the response data.
    ///
    /// # Errors
    /// Returns a [`SecRequestError`] if the request fails for any reason,
    /// including network errors, timeouts, or response parsing failures.
    async fn execute_request(&self, request: SecRequest) -> Result<SecResponse, SecRequestError>;

    /// Creates a boxed clone of this HTTP client.
    ///
    /// This method enables cloning of trait objects, which is necessary for
    /// [`SecClient`] to implement the [`Clone`] trait.
    ///
    /// # Returns
    /// Returns a new boxed instance of the HTTP client.
    fn clone_box(&self) -> Box<dyn HttpClient>;
}

/// Enables cloning of boxed [`HttpClient`] trait objects.
impl Clone for Box<dyn HttpClient> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
