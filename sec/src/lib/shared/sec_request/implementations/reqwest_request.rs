//! # Reqwest Request Implementation
//!
//! This module provides a concrete implementation of the [`InnerRequest`] trait using
//! the `reqwest` library.
//!
//! ## Types
//! - [`ReqwestRequest`]: Wrapper around `reqwest::Request` that implements [`InnerRequest`].
//!
//! ## See Also
//! - [`super::super::traits::inner_request::InnerRequest`]: The trait this implementation fulfills.
//! - [`reqwest::Request`]: The underlying HTTP request.

use reqwest::{Method, Request, Url};

use super::super::traits::inner_request::InnerRequest;

/// Implementation of [`InnerRequest`] using the `reqwest` library.
///
/// `ReqwestRequest` wraps a `reqwest::Request` and implements the [`InnerRequest`]
/// trait, enabling it to be used with [`super::super::SecRequest`] for dependency injection.
#[derive(Debug)]
pub struct ReqwestRequest {
    inner: Request,
}

impl ReqwestRequest {
    /// Creates a new `ReqwestRequest` wrapping the provided `reqwest::Request`.
    ///
    /// # Arguments
    /// * `request` - The `reqwest::Request` to wrap.
    ///
    /// # Returns
    /// Returns a new `ReqwestRequest` instance.
    #[must_use]
    pub const fn new(request: Request) -> Self {
        Self { inner: request }
    }

    /// Returns a reference to the underlying `reqwest::Request`.
    #[must_use]
    pub const fn request(&self) -> &Request {
        &self.inner
    }

    /// Creates a new GET request for the given URL.
    ///
    /// # Arguments
    /// * `url` - The URL for the request.
    ///
    /// # Returns
    /// Returns a new `ReqwestRequest` instance.
    #[must_use]
    pub fn get(url: Url) -> Self {
        Self {
            inner: Request::new(Method::GET, url),
        }
    }

    /// Consumes self and returns the inner `reqwest::Request`.
    #[must_use]
    pub fn into_inner(self) -> Request {
        self.inner
    }
}

impl InnerRequest for ReqwestRequest {
    fn url(&self) -> &Url {
        self.inner.url()
    }

    fn try_clone(&self) -> Option<Self> {
        self.inner.try_clone().map(|req| Self { inner: req })
    }
}

impl Clone for ReqwestRequest {
    fn clone(&self) -> Self {
        self.try_clone()
            .expect("ReqwestRequest should always be cloneable")
    }
}
