use async_trait::async_trait;

use crate::shared::response::InnerResponse;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FakeInnerResponse {
    pub url: String,
    pub body: String,
    pub headers: String,
    pub status_code: u16,
    pub content_type: String,
}

impl FakeInnerResponse {
    pub fn new(
        url: String,
        body: String,
        headers: String,
        status_code: u16,
        content_type: String,
    ) -> Self {
        Self {
            url,
            body,
            headers,
            status_code,
            content_type,
        }
    }
}

#[async_trait]
impl InnerResponse for FakeInnerResponse {
    type Url = String;
    type Body = String;
    type Headers = String;
    type StatusCode = u16;
    type ContentType = String;
    type Error = String;

    fn url(&self) -> &Self::Url {
        &self.url
    }

    fn headers(&self) -> &Self::Headers {
        &self.headers
    }

    async fn body(self) -> Result<Self::Body, Self::Error> {
        Ok(self.body)
    }

    fn status_code(&self) -> Self::StatusCode {
        self.status_code
    }

    fn content_type(&self) -> Self::ContentType {
        self.content_type.clone()
    }
}
