use std::fmt::Debug;

use async_trait::async_trait;

/// The transport-level HTTP response, decoupled from any HTTP crate.
///
/// Abstracts over a concrete response type so the domain layer doesn't depend on a specific
/// transport. Implemented for the `reqwest` response type and the test fakes;
/// [`SecResponse`](super::SecResponse) builds on top of it.
///
/// # Associated Types
///
/// Each implementor binds these to its transport's concrete types, which is what lets the domain
/// layer stay independent of any specific HTTP crate:
///
/// - `Url`: The type representing the response's URL.
/// - `Headers`: The type representing the response's headers.
/// - `Body`: The type representing the response's body.
/// - `StatusCode`: The type representing the response's HTTP status code.
/// - `ContentType`: The type representing the response's content type.
/// - `Error`: The error type returned when reading the body fails.
#[async_trait]
pub trait InnerResponse: Send + Sync + Debug {
    /// The type representing the response's URL.
    type Url;

    /// The type representing the response's headers.
    type Headers;

    /// The type representing the response's body.
    type Body;

    /// The type representing the response's HTTP status code.
    type StatusCode;

    /// The type representing the response's content type.
    type ContentType;

    /// The error type returned when reading the body fails.
    type Error;

    /// Returns a reference to the response's URL.
    fn url(&self) -> &Self::Url;

    /// Returns a reference to the response's headers.
    fn headers(&self) -> &Self::Headers;

    /// Consumes the response and reads its body.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the body cannot be read.
    async fn body(self) -> Result<Self::Body, Self::Error>;

    /// Returns the response's HTTP status code.
    fn status_code(&self) -> Self::StatusCode;

    /// Returns the response's content type.
    fn content_type(&self) -> Self::ContentType;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::shared::response::InnerResponse;
    use crate::tests::fixtures::sample_response::sample_inner_response::FakeInnerResponse;

    #[test]
    fn should_create_fake_response_with_specified_url_when_using_constructor() {
        let url = String::from("https://example.com");
        let body = String::from("This is a fake response body.");
        let headers = String::from("This is a fake response header.");
        let status_code = 200;
        let content_type = String::from("application/json");

        let expected_result = FakeInnerResponse {
            url: String::from("https://example.com"),
            body: String::from("This is a fake response body."),
            headers: String::from("This is a fake response header."),
            status_code: 200,
            content_type: String::from("application/json"),
        }
        .url()
        .clone();

        let result = FakeInnerResponse::new(url, body, headers, status_code, content_type)
            .url()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_create_fake_response_with_specified_body_when_using_constructor() {
        let url = String::from("https://example.com");
        let body = String::from("This is a fake response body.");
        let headers = String::from("This is a fake response header.");
        let status_code = 200;
        let content_type = String::from("application/json");

        let expected_result = FakeInnerResponse {
            url: String::from("https://example.com"),
            body: String::from("This is a fake response body."),
            headers: String::from("This is a fake response header."),
            status_code: 200,
            content_type: String::from("application/json"),
        }
        .body()
        .await
        .expect("A hardcoded FakeInnerResponse body should always succeed");

        let result = FakeInnerResponse::new(url, body, headers, status_code, content_type)
            .body()
            .await
            .expect("A hardcoded FakeInnerResponse body should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_fake_response_with_specified_status_code_when_using_constructor() {
        let url = String::from("https://example.com");
        let body = String::from("This is a fake response body.");
        let headers = String::from("This is a fake response header.");
        let status_code = 200;
        let content_type = String::from("application/json");

        let expected_result = FakeInnerResponse {
            url: String::from("https://example.com"),
            body: String::from("This is a fake response body."),
            headers: String::from("This is a fake response header."),
            status_code: 200,
            content_type: String::from("application/json"),
        }
        .status_code()
        .clone();

        let result = FakeInnerResponse::new(url, body, headers, status_code, content_type)
            .status_code()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_fake_response_with_specified_content_type_when_using_constructor() {
        let url = String::from("https://example.com");
        let body = String::from("This is a fake response body.");
        let headers = String::from("This is a fake response header.");
        let status_code = 200;
        let content_type = String::from("application/json");

        let expected_result = FakeInnerResponse {
            url: String::from("https://example.com"),
            body: String::from("This is a fake response body."),
            headers: String::from("This is a fake response header."),
            status_code: 200,
            content_type: String::from("application/json"),
        }
        .content_type()
        .clone();

        let result = FakeInnerResponse::new(url, body, headers, status_code, content_type)
            .content_type()
            .clone();

        assert_eq!(result, expected_result);
    }
}
