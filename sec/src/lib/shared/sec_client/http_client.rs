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
/// and testing. Any HTTP client implementation can be used with [`super::SecClient`] as
/// long as it implements this trait.
#[async_trait]
pub trait HttpClient: Send + Sync + Clone {
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
}
