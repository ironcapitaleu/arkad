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
//! - [`routes`]: The versioned routes and their spec.
//! - [`shared`]: The domain types, which depend on no route.

use axum::middleware::from_fn;
use axum::routing::get;
use axum::{Extension, Router};

use crate::routes::v1::openapi::SpecJson;

pub mod config;
pub mod error;
pub mod health;
pub mod logging;
pub mod middleware;
pub mod routes;
pub mod shared;

/// Builds the router with every route of the API.
///
/// The router answers a path that no route matches with a `404` problem body.
pub fn router() -> Router {
    let (v1, spec) = routes::v1::router().split_for_parts();
    let v1 = v1.layer(Extension(SpecJson::new(&spec)));
    Router::new()
        .route("/health", get(health::health))
        .merge(v1)
        .fallback(error::not_found)
        .layer(from_fn(middleware::problem::render_problem))
        .layer(from_fn(middleware::request_id::assign_request_id))
}

/// Returns the spec of every `/v1` route.
#[must_use]
pub fn openapi() -> utoipa::openapi::OpenApi {
    routes::v1::router().into_openapi()
}
