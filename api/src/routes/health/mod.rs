//! # Health
//!
//! Provides the [`health`] handler of `GET /health`, which uptime checks call.

use axum::Json;
use serde::Serialize;

/// The body of a `GET /health` response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HealthStatus {
    /// Always `"ok"`, because the handler only answers while the process serves requests.
    pub status: &'static str,
}

/// Answers `GET /health` with `200` and `{"status":"ok"}`.
// axum requires an async handler. This one has nothing to await.
#[allow(clippy::unused_async)]
pub async fn health() -> Json<HealthStatus> {
    Json(HealthStatus { status: "ok" })
}
