use async_trait::async_trait;
use reqwest::header::HeaderMap;
use reqwest::{Response, StatusCode, Url};

use super::super::traits::InnerResponse;

#[async_trait]
impl InnerResponse for Response {
    type Url = Url;
    type Headers = HeaderMap;
    type Body = String;
    type StatusCode = StatusCode;
    type ContentType = String;
    type Error = reqwest::Error;

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
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string()
    }

    /// Consumes the response and returns the body as a UTF-8 string.
    async fn body(self) -> Result<Self::Body, Self::Error> {
        self.text().await
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use reqwest::{Method, Request, Url};

    use crate::shared::request::traits::inner::InnerRequest;
    #[test]
    fn should_create_same_request_with_same_method_when_using_trait_constructor() {
        let method = Method::GET;
        let url = Url::parse("https://example.com").expect("Hardcoded URL should always be valid");

        let expected_result = Request::new(method.clone(), url.clone()).method().clone();

        let result = <Request as InnerRequest>::new(method, url).method().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_same_request_with_same_url_when_using_trait_constructor() {
        let method = Method::GET;
        let url = Url::parse("https://example.com").expect("Hardcoded URL should always be valid");

        let expected_result = Request::new(method.clone(), url.clone()).url().clone();

        let result = <Request as InnerRequest>::new(method, url).url().clone();

        assert_eq!(result, expected_result);
    }
}
