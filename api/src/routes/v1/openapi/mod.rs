//! # Spec
//!
//! Provides the [`openapi_json`] handler of `GET /v1/openapi.json` and the [`SpecJson`] it
//! serves.

use axum::Extension;
use axum::body::Bytes;
use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use utoipa::openapi::OpenApi;

/// The path of the spec.
pub const OPENAPI_JSON_PATH: &str = "/v1/openapi.json";

/// The spec as pretty-printed JSON text, which the router serves at [`OPENAPI_JSON_PATH`].
///
/// The router adds it to every request as an extension. A clone shares the text and copies none
/// of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecJson(Bytes);

impl SpecJson {
    /// Creates a new [`SpecJson`] from `spec`.
    ///
    /// # Panics
    ///
    /// Panics if `serde_json` fails to serialize the spec. The spec holds only maps, lists, and
    /// strings, so serialization does not fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use api::routes::v1::openapi::SpecJson;
    ///
    /// let spec = api::openapi();
    ///
    /// let expected_result = Some(b'{');
    ///
    /// let result = SpecJson::new(&spec).as_bytes().first().copied();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn new(spec: &OpenApi) -> Self {
        let text = spec.to_pretty_json().expect(
            "Given a spec of maps, lists, and strings, serializing it should always succeed",
        );
        Self(Bytes::from(text))
    }

    /// Returns the JSON text as bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Answers `GET /v1/openapi.json` with the spec of `/v1`.
#[utoipa::path(
    get,
    path = "/openapi.json",
    tag = "openapi",
    responses(
        (status = 200, description = "The spec of `/v1`, in OpenAPI 3.1", content_type = "application/json", body = Object),
    ),
)]
// axum requires an async handler. This one has nothing to await.
#[allow(clippy::unused_async)]
pub async fn openapi_json(Extension(spec): Extension<SpecJson>) -> impl IntoResponse {
    ([(CONTENT_TYPE, "application/json")], spec.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_spec_json() {
        implements_auto_traits::<SpecJson>();
    }
}
