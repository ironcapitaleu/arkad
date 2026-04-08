use std::fmt::Debug;

use async_trait::async_trait;
use serde_json::Value;

use super::InnerResponse;

/// A trait defining the interface of an HTTP response. This is used to decouple from third party libraries.
#[async_trait]
pub trait SecResponse: Send + Sync + Debug {
    /// This type represents the inner response type of the HTTP client library that we use to make requests to the SEC endpoints.
    type Inner: InnerResponse;

    /// This type represents the syntactical and semantic errors that can occur when processing a response to an SEC API request.
    type Error;

    fn inner(&self) -> &Self::Inner;

    /// Consumes the response and returns the body as a valid JSON value.
    async fn body(self) -> Result<Value, Self::Error>;

    fn url(&self) -> &<Self::Inner as InnerResponse>::Url {
        self.inner().url()
    }

    fn headers(&self) -> &<Self::Inner as InnerResponse>::Headers {
        self.inner().headers()
    }

    fn status_code(&self) -> <Self::Inner as InnerResponse>::StatusCode {
        self.inner().status_code()
    }

    fn content_type(&self) -> <Self::Inner as InnerResponse>::ContentType {
        self.inner().content_type()
    }
}
