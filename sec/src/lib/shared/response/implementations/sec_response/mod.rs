//! # SEC Response
//!
//! Provides the [`SecResponse`], the concrete [`SecResponse`](crate::shared::response::SecResponse)
//! returned by the SEC API.
//!
//! ## Modules
//!
//! - [`body_digest`]: The [`BodyDigest`] backing efficient `Hash`/`Ord`.
//! - [`error`]: The [`InvalidSecResponse`] error raised during validation.

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use async_trait::async_trait;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::status_code::StatusCode;
use crate::shared::url::Url;

use super::super::traits::SecResponse as SecResponseTrait;

use self::body_digest::BodyDigest;
use self::error::{ErrorReason, InvalidSecResponse};

pub mod body_digest;
pub mod error;

/// A validated SEC API response.
///
/// Only ever constructed once the HTTP response clears validation — a 2xx status, a JSON content
/// type, and a syntactically valid JSON body — so code holding a `SecResponse` can trust those
/// invariants. Built from a raw response via
/// [`from_inner`](crate::shared::response::SecResponse::from_inner), or from parts via
/// [`SecResponse::from_parts`].
#[derive(Debug, Clone)]
pub struct SecResponse {
    url: Url,
    headers: Headers,
    content_type: ContentType,
    status_code: StatusCode,
    body: serde_json::Value,
    body_digest: BodyDigest,
}

impl PartialEq for SecResponse {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
            && self.content_type == other.content_type
            && self.status_code == other.status_code
            && self.body_digest == other.body_digest
    }
}

impl Eq for SecResponse {}

impl std::hash::Hash for SecResponse {
    // `Headers` and `serde_json::Value` do not implement `Hash`.
    // The body is represented by its precomputed `BodyDigest`.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.content_type.hash(state);
        self.status_code.hash(state);
        self.body_digest.hash(state);
    }
}

impl PartialOrd for SecResponse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SecResponse {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.url
            .cmp(&other.url)
            .then_with(|| self.content_type.cmp(&other.content_type))
            .then_with(|| self.status_code.cmp(&other.status_code))
            .then_with(|| self.body_digest.cmp(&other.body_digest))
    }
}

impl SecResponse {
    /// Creates a `SecResponse` directly from already-validated parts, skipping HTTP validation.
    ///
    /// Unlike [`from_inner`](SecResponseTrait::from_inner), the caller is responsible for ensuring
    /// the parts represent a valid SEC response. The body digest is taken from `body.to_string()`
    /// (re-serialized JSON), which may differ from the raw text `from_inner` hashes due to
    /// whitespace or key ordering — harmless, since the two paths are never applied to the same data.
    #[must_use]
    pub fn from_parts(
        url: Url,
        headers: Headers,
        content_type: ContentType,
        status_code: StatusCode,
        body: serde_json::Value,
    ) -> Self {
        let body_digest = BodyDigest::from_body_text(&body.to_string());
        Self {
            url,
            headers,
            content_type,
            status_code,
            body,
            body_digest,
        }
    }

    /// Returns the precomputed body digest.
    #[must_use]
    pub const fn body_digest(&self) -> BodyDigest {
        self.body_digest
    }
}

impl Serialize for SecResponse {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("SecResponse", 4)?;
        state.serialize_field("url", &self.url.to_string())?;
        state.serialize_field("status_code", &self.status_code.to_string())?;
        state.serialize_field("content_type", &self.content_type.to_string())?;
        state.serialize_field("headers", &self.headers)?;
        state.end()
    }
}

impl Display for SecResponse {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {}", self.status_code, self.url)
    }
}

#[async_trait]
impl SecResponseTrait for SecResponse {
    type Inner = reqwest::Response;
    type Url = Url;
    type Headers = Headers;
    type StatusCode = StatusCode;
    type ContentType = ContentType;
    type Error = InvalidSecResponse;

    async fn from_inner(inner: Self::Inner) -> Result<Self, Self::Error> {
        let url = Url::from(inner.url().clone());
        let status_code = StatusCode::from(inner.status());
        let raw_headers: HashMap<String, String> = inner
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let headers = Headers::new(raw_headers);
        let content_type = headers.content_type().clone();

        if !status_code.is_success() {
            return Err(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
                status_code,
            }));
        }

        if content_type != ContentType::Json {
            return Err(InvalidSecResponse::new(ErrorReason::InvalidContentType {
                content_type,
            }));
        }

        let body_text = inner.text().await.map_err(|e| {
            InvalidSecResponse::new(ErrorReason::FailedBodyRead {
                details: e.to_string(),
            })
        })?;

        let body_digest = BodyDigest::from_body_text(&body_text);

        let body = serde_json::from_str(&body_text).map_err(|e| {
            InvalidSecResponse::new(ErrorReason::InvalidBody {
                details: e.to_string(),
            })
        })?;

        Ok(Self {
            url,
            headers,
            content_type,
            status_code,
            body,
            body_digest,
        })
    }

    fn url(&self) -> &Self::Url {
        &self.url
    }

    fn headers(&self) -> &Self::Headers {
        &self.headers
    }

    fn status_code(&self) -> Self::StatusCode {
        self.status_code
    }

    fn content_type(&self) -> Self::ContentType {
        self.content_type.clone()
    }

    fn body(&self) -> &serde_json::Value {
        &self.body
    }
}
