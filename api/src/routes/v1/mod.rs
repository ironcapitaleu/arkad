//! # V1
//!
//! Provides the routes under `/v1` and the spec that describes them.
//!
//! Every route registers through [`OpenApiRouter`] and the `routes!` macro. The macro adds the
//! route to the router and to the spec in one step, so the two list the same routes.
//!
//! ## Modules
//!
//! - [`concepts`]: `GET /v1/concepts`, the list of canonical concepts.
//! - [`openapi`]: `GET /v1/openapi.json`, the spec.

use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::error::Problem;

pub mod concepts;
pub mod openapi;

/// The path prefix of every route in this module.
pub const V1_PREFIX: &str = "/v1";

/// The parts of the spec that no route adds: the `info` object and the [`Problem`]
/// schema.
#[derive(OpenApi)]
#[openapi(components(schemas(Problem)))]
pub struct V1Spec;

/// Builds the router of every `/v1` route together with its spec.
pub fn router() -> OpenApiRouter {
    let routes = OpenApiRouter::new()
        .routes(routes!(concepts::list_concepts))
        .routes(routes!(openapi::openapi_json));
    OpenApiRouter::with_openapi(V1Spec::openapi()).nest(V1_PREFIX, routes)
}
