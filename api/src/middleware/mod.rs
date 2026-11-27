//! # Middleware
//!
//! Provides the middleware that the router runs on every request, and the extractors that check a
//! request before a handler runs.
//!
//! ## Modules
//!
//! - [`authentication`]: Checks the credential of a request before a handler runs.
//! - [`problem`]: Writes the problem body of an error response.
//! - [`request_id`]: Gives every request an ID and returns it in the `X-Request-Id` header.

pub mod authentication;
pub mod problem;
pub mod request_id;
