//! # Routes
//!
//! Provides the router of the crate and the routes it joins.
//!
//! ## Modules
//!
//! - [`health`]: The `GET /health` endpoint that uptime checks call.
//! - [`v1`]: The routes under `/v1` and the spec that describes them.

use axum::middleware::from_fn;
use axum::routing::get;
use axum::{Extension, Router};
use utoipa_swagger_ui::SwaggerUi;

use crate::config::InternalDocs;
use crate::routes::v1::openapi::{OPENAPI_JSON_PATH, SpecJson};
use crate::{error, middleware};

pub mod health;
pub mod v1;

/// The path of the Swagger UI page.
pub const INTERNAL_DOCS_PATH: &str = "/internal/docs";

/// Builds the router with `GET /health` and every `/v1` route.
///
/// The router answers a path that no route matches with a `404` problem body. If
/// `internal_docs` is [`InternalDocs::Enabled`], it serves the Swagger UI at
/// [`INTERNAL_DOCS_PATH`].
pub fn router(internal_docs: InternalDocs) -> Router {
    let (v1, spec) = v1::router().split_for_parts();
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
        .layer(from_fn(middleware::problem::render_problem))
        .layer(from_fn(middleware::request_id::assign_request_id))
}
