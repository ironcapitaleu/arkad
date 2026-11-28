//! # Routes
//!
//! Provides the router of the crate, the routes it joins, and the [`AppState`] it gives them.
//!
//! ## Modules
//!
//! - [`health`]: The `GET /health` endpoint that uptime checks call.
//! - [`v1`]: The routes under `/v1` and the spec that describes them.

use std::sync::Arc;

use axum::extract::FromRef;
use axum::middleware::from_fn;
use axum::routing::get;
use axum::{Extension, Router};
use utoipa_swagger_ui::SwaggerUi;

use crate::config::InternalDocs;
use crate::routes::v1::openapi::{OPENAPI_JSON_PATH, SpecJson};
use crate::traits::token_verifier::TokenVerifier;
use crate::{error, middleware};

pub mod health;
pub mod v1;

/// The path of the Swagger UI page.
pub const INTERNAL_DOCS_PATH: &str = "/internal/docs";

/// The state that the router gives its handlers and extractors.
///
/// Each port in the state has its own [`FromRef`] impl, so an extractor asks for the port and not
/// for [`AppState`]. So a test can pass a state that holds only the ports it needs.
#[derive(Clone)]
pub struct AppState {
    token_verifier: Arc<dyn TokenVerifier>,
}

impl AppState {
    /// Creates a new [`AppState`] that checks login tokens with `token_verifier`.
    #[must_use]
    pub fn new(token_verifier: Arc<dyn TokenVerifier>) -> Self {
        Self { token_verifier }
    }
}

impl FromRef<AppState> for Arc<dyn TokenVerifier> {
    /// Returns the [`TokenVerifier`] of the [`AppState`].
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.token_verifier)
    }
}

/// Builds the router with `GET /health` and every `/v1` route, and gives the routes `state`.
///
/// The router answers a path that no route matches with a `404` problem body. If
/// `internal_docs` is [`InternalDocs::Enabled`], it serves the Swagger UI at
/// [`INTERNAL_DOCS_PATH`].
pub fn router(internal_docs: InternalDocs, state: AppState) -> Router {
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
        .with_state(state)
}
