//! # API
//!
//! Provides the axum router, the configuration, and the logger of the `api` binary.
//!
//! The binary serves the HTTP API for company fundamentals. This library holds
//! everything except the start of the process, so integration tests can call the router without a
//! network socket.
//!
//! ## Modules
//!
//! - [`config`]: The configuration read from environment variables.
//! - [`error`]: The errors that a client sees, sent as RFC 9457 problem bodies.
//! - [`logging`]: The JSON logger and the events of the binary.
//! - [`middleware`]: The middleware that the router runs on every request.
//! - [`routes`]: The router of the crate, with `GET /health` and the versioned routes.
//! - [`shared`]: The domain types, which depend on no route.
//! - [`traits`]: The traits of the API.

use axum::Router;

use crate::config::InternalDocs;

pub mod config;
pub mod error;
pub mod logging;
pub mod middleware;
pub mod routes;
pub mod shared;
pub mod traits;

pub use routes::INTERNAL_DOCS_PATH;

/// Builds the router of the crate. See [`routes::router`].
pub fn router(internal_docs: InternalDocs) -> Router {
    routes::router(internal_docs)
}

/// Returns the spec of every `/v1` route.
#[must_use]
pub fn openapi() -> utoipa::openapi::OpenApi {
    routes::v1::router().into_openapi()
}
