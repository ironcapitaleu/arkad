//! # Fake HTTP Client Fixtures
//!
//! This module provides fake HTTP client implementations for testing purposes.
//!
//! ## Types
//! - [`ValidFakeHttpClient`]: Always returns successful responses with JSON content.
//! - [`InvalidFakeHttpClient`]: Always returns network error responses.

pub mod invalid_fake_http_client;
pub mod valid_fake_http_client;

pub use invalid_fake_http_client::InvalidFakeHttpClient;
pub use valid_fake_http_client::ValidFakeHttpClient;
