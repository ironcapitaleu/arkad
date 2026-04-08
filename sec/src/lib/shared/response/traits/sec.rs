use std::fmt::Debug;

use async_trait::async_trait;

use super::InnerResponse;

/// A trait defining the interface of an HTTP response. This is used to decouple from third party libraries.
#[async_trait]
pub trait SecResponse: Send + Sync + Debug + Sized {
    /// The HTTP response type used to construct this `SecResponse`.
    type Inner: InnerResponse;

    /// The type representing the URL of the response.
    type Url;

    /// The type representing the headers of the response.
    type Headers;

    /// The type representing the HTTP status code of the response.
    type StatusCode;

    /// The type representing the content type of the response.
    type ContentType;

    /// This type represents the syntactical and semantic errors that can occur when processing a response to an SEC API request.
    type Error;

    /// Consumes the inner response and constructs a new `SecResponse` instance.
    ///
    /// This method is asynchronous because it will read the inner response body which might involve an asynchronous operation.
    async fn new(inner: Self::Inner) -> Result<Self, Self::Error>;

    /// Returns the URL of the response.
    fn url(&self) -> &Self::Url;

    /// Returns the headers of the response.
    fn headers(&self) -> &Self::Headers;

    /// Returns the HTTP status code of the response.
    fn status_code(&self) -> Self::StatusCode;

    /// Returns the content type of the response.
    fn content_type(&self) -> Self::ContentType;

    /// Returns a reference to the response body as a valid JSON value.
    fn body(&self) -> &serde_json::Value;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::shared::response::traits::SecResponse;

    use crate::tests::fixtures::sample_response::{
        sample_inner_response::FakeInnerResponse, sample_sec_response::FakeSecResponse,
    };

    #[tokio::test]
    async fn should_create_fake_sec_response_with_empty_json_object_body_when_inner_response_body_is_empty_json_object()
     {
        let fake_inner_response = FakeInnerResponse {
            url: String::from("https://example.com/"),
            body: String::from("{}"),
            headers: String::new(),
            status_code: 200,
            content_type: String::from("application/json"),
        };

        let expected_result = String::from("{}");

        let result = FakeSecResponse::new(fake_inner_response)
            .await
            .expect("Failed to create FakeSecResponse")
            .body()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_create_fake_sec_response_with_valid_json_object_body_when_inner_response_body_is_valid_json()
     {
        let fake_inner_response = FakeInnerResponse {
            url: String::from("https://example.com/"),
            body: String::from("{\"key\": \"value\"}"),
            headers: String::new(),
            status_code: 200,
            content_type: String::from("application/json"),
        };

        let expected_result = String::from("{\"key\":\"value\"}");

        let result = FakeSecResponse::new(fake_inner_response)
            .await
            .expect("Failed to create FakeSecResponse")
            .body()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    #[should_panic(expected = "Should fail creating FakeSecResponse with empty string as body")]
    async fn should_fail_creating_fake_sec_response_with_empty_string_as_body() {
        let fake_inner_response = FakeInnerResponse {
            url: String::from("https://example.com/"),
            body: String::new(),
            headers: String::new(),
            status_code: 200,
            content_type: String::from("application/json"),
        };

        let _result = FakeSecResponse::new(fake_inner_response)
            .await
            .expect("Should fail creating FakeSecResponse with empty string as body");
    }

    #[tokio::test]
    #[should_panic(
        expected = "Should fail creating FakeSecResponse with invalid json string as body"
    )]
    async fn should_fail_creating_fake_sec_response_with_invalid_json_string_as_body() {
        let fake_inner_response = FakeInnerResponse {
            url: String::from("https://example.com/"),
            body: String::from("{Some invalid json: mah man}"),
            headers: String::new(),
            status_code: 200,
            content_type: String::from("application/json"),
        };

        let _result = FakeSecResponse::new(fake_inner_response)
            .await
            .expect("Should fail creating FakeSecResponse with invalid json string as body");
    }
}
