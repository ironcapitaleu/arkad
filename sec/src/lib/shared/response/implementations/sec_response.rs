use super::super::traits::SecResponse as SecResponseTrait;
use super::content_type::ContentType;

pub struct SecResponse {
    inner: reqwest::Response,
}

#[async_trait]
impl SecResponseTrait for SecResponse {
    type Url = Url;
    type Headers = HeaderMap;
    type Body = String;
    type StatusCode = StatusCode;
    type ContentType = ContentType;
    type Error = reqwest::Error; // TODO: Placeholder for now. The `Transform` `SuperState` might be adding different error types when semantically checking the response contents.

    /// Returns the URL endpoint of the HTTP request.
    fn url(&self) -> &Self::Url {
        self.url()
    }

    /// Returns the headers of the HTTP response.
    fn headers(&self) -> &Self::Headers {
        self.headers()
    }

    /// Returns the HTTP status code of the response.
    fn status_code(&self) -> Self::StatusCode {
        self.status()
    }

    /// Returns the content type of the HTTP response.
    ///
    /// Returns an empty string if the `Content-Type` header is absent or contains invalid UTF-8.
    fn content_type(&self) -> Self::ContentType {
        self.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map_or_else(|| ContentType::Unknown, ContentType::from_content_type)
    }

    /// Consumes the response and returns the body as a UTF-8 string.
    async fn body(self) -> Result<Self::Body, Self::Error> {
        self.text().await
    }
}
