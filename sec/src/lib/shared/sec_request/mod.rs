//! # SEC Request Utilities
//!
//! This module provides the [`SecRequest`] type and related utilities for creating HTTP requests to the
//! SEC API endpoints. It is used throughout the SEC state machine library to ensure that
//! SEC API requests are constructed in a consistent and robust manner.
//!
//! ## Modules
//! - [`constants`]: Constants related to SEC API endpoints, such as URL prefixes and suffixes.
//!
//! ## Types
//! - [`SecRequest`]: Strongly-typed wrapper for HTTP requests to SEC API endpoints, with methods for creating requests for specific CIKs.
//!
//! ## Usage
//! The [`SecRequest`] type is used by state machine implementations to create properly formatted HTTP requests
//! to SEC API endpoints. The module provides methods for constructing requests for specific CIKs and ensures
//! that all requests follow the correct URL format and HTTP method.
//!
//! ## See Also
//! - [`crate::shared`]: Shared domain types and utilities used across the SEC state machine library.
//! - [`crate::shared::cik`]: CIK utilities that work with this module to create targeted requests.
//! - [`crate::shared::sec_client`]: SEC client utilities for executing these requests.

pub mod constants;

use constants::{SEC_CIK_BERKSHIRE_HATHAWAY_URL, SEC_REQUEST_URL_PREFIX, SEC_REQUEST_URL_SUFFIX};

use reqwest::Request;

use crate::shared::cik::Cik;

/// Strongly-typed wrapper for HTTP requests to SEC API endpoints.
///
/// The `SecRequest` type ensures that HTTP requests to SEC API endpoints are properly formatted
/// and follow the correct URL structure. Use [`SecRequest::new`] to construct a request for a specific CIK.
#[derive(Debug)]
pub struct SecRequest {
    pub inner: reqwest::Request,
}

impl SecRequest {
    #[must_use]
    /// Creates a new `SecRequest` for a given CIK.
    ///
    /// # Panics
    /// Panics if the URL cannot be parsed, which should not happen with hardcoded URLs.
    pub fn new(cik: &Cik) -> Self {
        let url = format!("{SEC_REQUEST_URL_PREFIX}{cik}{SEC_REQUEST_URL_SUFFIX}");
        Self {
            inner: Request::new(
                reqwest::Method::GET,
                reqwest::Url::parse(&url).expect("Hardcoded URL should always be valid."),
            ),
        }
    }

    pub const fn request(&self) -> &Request {
        &self.inner
    }
}

impl Clone for SecRequest {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.try_clone().expect("Failed to clone Request"),
        }
    }
}

impl PartialEq for SecRequest {
    fn eq(&self, other: &Self) -> bool {
        self.inner.url() == other.inner.url()
    }
}

impl PartialOrd for SecRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.inner.url().cmp(other.inner.url()))
    }
}

impl Ord for SecRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.url().cmp(other.inner.url())
    }
}

impl Eq for SecRequest {}

impl std::hash::Hash for SecRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.url().hash(state);
    }
}

impl Default for SecRequest {
    fn default() -> Self {
        Self {
            inner: Request::new(
                reqwest::Method::GET,
                reqwest::Url::parse(SEC_CIK_BERKSHIRE_HATHAWAY_URL)
                    .expect("Hardcoded URL should always be valid."),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_sec_request_when_valid_cik_is_provided() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should be valid");
        let expected_url = "https://data.sec.gov/submissions/CIK1234567890.json";
        let result = SecRequest::new(&cik);

        assert_eq!(result.inner.url().as_str(), expected_url);
        assert_eq!(result.inner.method(), &reqwest::Method::GET);
    }

    #[test]
    fn should_create_sec_request_with_zero_padded_cik_when_short_cik_is_provided() {
        let cik = Cik::new("123456789").expect("Hardcoded CIK should be valid");
        let expected_url = "https://data.sec.gov/submissions/CIK0123456789.json";
        let result = SecRequest::new(&cik);

        assert_eq!(result.inner.url().as_str(), expected_url);
    }

    #[test]
    fn should_create_default_sec_request_when_default_is_called() {
        let expected_url = SEC_CIK_BERKSHIRE_HATHAWAY_URL;
        let result = SecRequest::default();

        assert_eq!(result.inner.url().as_str(), expected_url);
        assert_eq!(result.inner.method(), &reqwest::Method::GET);
    }
}
