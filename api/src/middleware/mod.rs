//! # Middleware
//!
//! Provides the middleware that the router runs on every request.
//!
//! ## Modules
//!
//! - [`problem`]: Writes the problem body of an error response.
//! - [`request_id`]: Gives every request an ID and returns it in the `X-Request-Id` header.

pub mod problem;
pub mod request_id;
