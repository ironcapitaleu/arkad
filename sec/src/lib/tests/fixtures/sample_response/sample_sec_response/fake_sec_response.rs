use async_trait::async_trait;

use super::super::sample_inner_response::FakeInnerResponse;
use crate::shared::response::SecResponse;

#[derive(Debug, Clone, PartialEq)]
pub struct FakeSecResponse {
    pub inner: FakeInnerResponse,
    pub body: serde_json::Value,
}

#[async_trait]
impl SecResponse for FakeSecResponse {
    type Inner = FakeInnerResponse;
    type Url = String;
    type Headers = String;
    type StatusCode = u16;
    type ContentType = String;
    type Error = String;

    async fn from_inner(inner: Self::Inner) -> Result<Self, Self::Error> {
        let body = serde_json::from_str(&inner.body).map_err(|e| e.to_string())?;
        Ok(Self { inner, body })
    }

    fn url(&self) -> &Self::Url {
        &self.inner.url
    }

    fn headers(&self) -> &Self::Headers {
        &self.inner.headers
    }

    fn status_code(&self) -> Self::StatusCode {
        self.inner.status_code
    }

    fn content_type(&self) -> Self::ContentType {
        self.inner.content_type.clone()
    }

    fn body(&self) -> &serde_json::Value {
        &self.body
    }
}
