use std::collections::HashMap;

use async_trait::async_trait;

use crate::shared::content_type::ContentType;
use crate::shared::headers::Headers;
use crate::shared::status_code::StatusCode;

use super::super::traits::SecResponse as SecResponseTrait;

#[derive(Debug)]
pub struct SecResponse {
    url: String, // TODO: Change it later to own type that can be used across different response implementations.
    headers: Headers,
    content_type: ContentType,
    status_code: StatusCode,
    body: serde_json::Value,
}

#[async_trait]
impl SecResponseTrait for SecResponse {
    type Inner = reqwest::Response;
    type Url = String;
    type Headers = Headers;
    type StatusCode = StatusCode;
    type ContentType = ContentType;
    type Error = SecResponeError; // TODO: Placeholder for now. The `Transform` `SuperState` might be adding different error types when semantically checking the response contents.

    async fn from_inner(inner: Self::Inner) -> Result<Self, Self::Error> {
        let url = inner.url().to_string();
        let status_code = StatusCode::from(inner.status());
        let raw_headers: HashMap<String, String> = inner
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let headers = Headers::new(raw_headers);
        let content_type = headers.content_type().clone();
        let body_text = inner
            .text()
            .await
            .map_err(SecResponeError::FailedBodyRead)?;
        let body = serde_json::from_str(&body_text).map_err(SecResponeError::JsonParseError)?;

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

/// TODO: Placeholder for now. The `Transform` `SuperState` might be adding different error types when semantically checking the response contents.
/// TODO: Use Field-based Enum Variants instead of Tuple-based Variants.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SecResponeError {
    #[error("Failed to read response body: {0}")]
    FailedBodyRead(reqwest::Error),
    #[error("Failed to parse response body as JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Wrong Response Status Code: {0}")]
    WrongStatusCode(u16),
    #[error("Unexpected Content-Type: {0}")]
    UnexpectedContentType(String),
}
