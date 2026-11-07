//! # Middleware
//!
//! Provides the middleware that the router runs on every request.
//!
//! ## Modules
//!
//! - [`request_id`]: Gives every request an ID and returns it in the `X-Request-Id` header.

pub mod request_id;
