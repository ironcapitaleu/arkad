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
//! - [`health`]: The `GET /health` endpoint that uptime checks call.
//! - [`logging`]: The JSON logger and the events of the binary.
//! - [`middleware`]: The middleware that the router runs on every request.

use axum::Router;
use axum::middleware::from_fn;
use axum::routing::get;

pub mod config;
pub mod error;
pub mod health;
pub mod logging;
pub mod middleware;

/// Builds the router with every route of the API.
///
/// The router answers a path that no route matches with a `404` problem body.
pub fn router() -> Router {
    Router::new()
        .route("/health", get(health::health))
        .fallback(error::not_found)
        .layer(from_fn(middleware::problem::render_problem))
        .layer(from_fn(middleware::request_id::assign_request_id))
}
