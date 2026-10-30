use std::collections::HashMap;

use async_trait::async_trait;

use super::super::sample_inner_response::FakeInnerResponse;
use crate::shared::headers::Headers;
use crate::shared::response::SecResponse;
use crate::shared::response::implementations::sec_response::{JsonBody, ResponseBody};

/// A fake SEC response that builds a body of kind `B` from the body of a [`FakeInnerResponse`].
///
/// It runs only the body checks of `B`, so a fake client can return it for any request kind.
#[derive(Debug, Clone)]
pub struct FakeSecResponse<B> {
    pub inner: FakeInnerResponse,
    pub body: B,
}

/// A [`FakeSecResponse`] with a JSON body.
pub type FakeJsonSecResponse = FakeSecResponse<JsonBody>;

#[async_trait]
impl<B: ResponseBody> SecResponse for FakeSecResponse<B> {
    type Inner = FakeInnerResponse;
    type Url = String;
    type Headers = String;
    type StatusCode = u16;
    type ContentType = String;
    type Body = B::Content;
    type Error = String;

    async fn from_inner(inner: Self::Inner) -> Result<Self, Self::Error> {
        let raw_body = inner.body.clone().into_bytes();
        let (body, _) =
            B::from_raw(&Headers::new(HashMap::new()), raw_body).map_err(|e| e.to_string())?;
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

    fn body(&self) -> &Self::Body {
        self.body.content()
    }
}
