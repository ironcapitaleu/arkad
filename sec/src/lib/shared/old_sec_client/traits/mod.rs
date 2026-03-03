//! # HTTP Client Traits
//!
//! This module contains trait definitions for HTTP client abstraction, enabling
//! dependency injection and flexible client implementations.

pub mod http_client;
pub mod sec_client;

pub use http_client::HttpClient;
pub use sec_client::SecClient;
