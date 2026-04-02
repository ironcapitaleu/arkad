use crate::shared::response::SecResponse;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FakeSecResponse {
    pub method: FakeMethod,
    pub url: String,
    pub body: String,
    pub header: String,
    pub status_code: u16,
    pub content_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FakeMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl FakeSecResponse {
    pub fn new(
        method: FakeMethod,
        url: String,
        body: String,
        header: String,
        status_code: u16,
        content_type: String,
    ) -> Self {
        Self {
            method,
            url,
            body,
            header,
            status_code,
            content_type,
        }
    }
}

impl SecResponse for FakeSecResponse {
    type Method = FakeMethod;
    type Url = String;
    type Body = String;
    type Header = String;
    type StatusCode = u16;
    type ContentType = String;
    type Error = String; // TODO:  check if needed, if not delete --- IGNORE ---

    fn method(&self) -> &Self::Method {
        &self.method
    }

    fn url(&self) -> &Self::Url {
        &self.url
    }

    fn headers(&self) -> &Self::Header {
        &self.header
    }

    fn body(&self) -> &Self::Body {
        &self.body
    }

    fn status_code(&self) -> &Self::StatusCode {
        &self.status_code
    }

    fn content_type(&self) -> &Self::ContentType {
        &self.content_type
    }
}
