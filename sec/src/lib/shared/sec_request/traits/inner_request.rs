//! # Inner Request Trait
//!
//! This module defines the [`InnerRequest`] trait which abstracts HTTP request functionality
//! for dependency injection.
//!
//! ## Types
//! - [`InnerRequest`]: Trait defining the interface for HTTP request operations.
//!
//! ## See Also
//! - [`super::super::SecRequest`]: Uses this trait for HTTP request abstraction.
//! - [`super::super::implementations::reqwest_request`]: Concrete implementation using reqwest.
use std::fmt::Debug;

use reqwest::Url;

/// Trait defining the interface for HTTP request operations.
///
/// This trait abstracts HTTP request functionality to enable dependency injection
/// and testing. Any HTTP request implementation can be used with [`super::super::SecRequest`] as
/// long as it implements this trait.
pub trait InnerRequest: Send + Sync + Clone + Debug {
    /// Returns the URL of the request.
    fn url(&self) -> &Url;

    /// Clones the request.
    ///
    /// This method is required because the underlying request type may not
    /// implement the standard `Clone` trait.
    fn try_clone(&self) -> Option<Self>
    where
        Self: Sized;
}
