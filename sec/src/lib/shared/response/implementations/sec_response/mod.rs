use std::collections::HashMap;
use std::fmt;

use async_trait::async_trait;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::status_code::StatusCode;
use crate::shared::url::Url;

use super::super::traits::SecResponse as SecResponseTrait;

use self::error::{ErrorReason, InvalidSecResponse};

pub mod error;

/// A validated SEC API response.
///
/// `SecResponse` is only constructed when the HTTP response meets all
/// validity requirements: a success status code (2xx), a JSON content type,
/// and a syntactically valid JSON body.
#[derive(Debug, Clone)]
pub struct SecResponse {
    url: Url,
    headers: Headers,
    content_type: ContentType,
    status_code: StatusCode,
    body: serde_json::Value,
}

impl PartialEq for SecResponse {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
            && self.headers == other.headers
            && self.status_code == other.status_code
            && self.body == other.body
    }
}

impl Eq for SecResponse {}

impl std::hash::Hash for SecResponse {
    // Deviation: `Headers` and `serde_json::Value` do not implement `Hash`,
    // so only `url`, `content_type`, and `status_code` are hashed.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.content_type.hash(state);
        self.status_code.hash(state);
    }
}

impl PartialOrd for SecResponse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SecResponse {
    // Deviation: `Headers`, `StatusCode`, and `serde_json::Value` do not implement
    // `Ord`, so ordering is based on URL only.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.url.cmp(&other.url)
    }
}

impl fmt::Display for SecResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
