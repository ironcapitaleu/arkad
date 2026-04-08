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
    type Error = String;

    async fn new(inner: Self::Inner) -> Result<Self, Self::Error> {
        let body = serde_json::from_str(&inner.body).map_err(|e| e.to_string())?;
        Ok(Self { inner, body })
    }

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn body(&self) -> &serde_json::Value {
        &self.body
    }
}
