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

use axum::middleware::from_fn;
use axum::routing::get;
use axum::{Extension, Router};
use utoipa_swagger_ui::SwaggerUi;

use crate::config::InternalDocs;
use crate::routes::v1::openapi::{OPENAPI_JSON_PATH, SpecJson};

pub mod config;
pub mod error;
pub mod health;
pub mod logging;
pub mod middleware;
pub mod routes;

/// The path of the Swagger UI page.
pub const INTERNAL_DOCS_PATH: &str = "/internal/docs";

/// Builds the router with every route of the API.
///
/// The router answers a path that no route matches with a `404` problem body. If
/// `internal_docs` is [`InternalDocs::Enabled`], it serves the Swagger UI at
/// [`INTERNAL_DOCS_PATH`].
pub fn router(internal_docs: InternalDocs) -> Router {
    let (v1, spec) = routes::v1::router().split_for_parts();
    let v1 = v1.layer(Extension(SpecJson::new(&spec)));
    let router = Router::new()
        .route("/health", get(health::health))
        .merge(v1);
    let router = match internal_docs {
        InternalDocs::Enabled => router.merge(
            SwaggerUi::new(INTERNAL_DOCS_PATH)
                .config(utoipa_swagger_ui::Config::new([OPENAPI_JSON_PATH])),
        ),
        InternalDocs::Disabled => router,
    };
    router
        .fallback(error::not_found)
        .layer(from_fn(error::render_problem))
        .layer(from_fn(middleware::request_id::assign_request_id))
}

/// Returns the spec of every `/v1` route.
#[must_use]
pub fn openapi() -> utoipa::openapi::OpenApi {
    routes::v1::router().into_openapi()
}
