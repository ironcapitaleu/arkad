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
//! - [`health`]: The `GET /health` endpoint that uptime checks call.
//! - [`logging`]: The JSON logger and the events of the binary.

use axum::Router;
use axum::routing::get;

pub mod config;
pub mod health;
pub mod logging;

/// Builds the router with every route of the API.
pub fn router() -> Router {
    Router::new().route("/health", get(health::health))
}
