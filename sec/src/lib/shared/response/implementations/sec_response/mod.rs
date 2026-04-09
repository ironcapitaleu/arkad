use std::collections::HashMap;

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
#[derive(Debug)]
pub struct SecResponse {
    url: Url,
    headers: Headers,
    content_type: ContentType,
    status_code: StatusCode,
    body: serde_json::Value,
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
