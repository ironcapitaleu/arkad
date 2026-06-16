use std::fmt::Debug;

use async_trait::async_trait;

use super::InnerResponse;

/// The domain-level SEC response: validated, typed access to an HTTP response's parts.
///
/// Sits above [`InnerResponse`], building from a raw response and exposing its URL, headers,
/// status, content type, and JSON body as validated domain types. Existing as a trait keeps the
/// library decoupled from any HTTP crate and lets a fake response stand in for the network.
///
/// # Associated Types
///
/// - `Inner`: The raw response consumed during construction, an [`InnerResponse`].
/// - `Url`: The response's URL type.
/// - `Headers`: The response's headers type.
/// - `StatusCode`: The response's HTTP status code type.
/// - `ContentType`: The response's content type.
/// - `Error`: The error returned when the response cannot be read or validated.
#[async_trait]
pub trait SecResponse: Send + Sync + Debug + Sized {
    /// The raw response consumed during construction.
    type Inner: InnerResponse;

    /// The response's URL type.
    type Url;

    /// The response's headers type.
    type Headers;

    /// The response's HTTP status code type.
    type StatusCode;

    /// The response's content type.
    type ContentType;

    /// The error returned when the response cannot be read or validated.
    type Error;

    /// Consumes a raw response and builds a validated `SecResponse`.
    ///
    /// Asynchronous because reading the response body may involve I/O.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the body cannot be read or the response fails SEC validation.
    async fn from_inner(inner: Self::Inner) -> Result<Self, Self::Error>;

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

        let result = FakeSecResponse::from_inner(fake_inner_response)
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

        let result = FakeSecResponse::from_inner(fake_inner_response)
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

        let _result = FakeSecResponse::from_inner(fake_inner_response)
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

        let _result = FakeSecResponse::from_inner(fake_inner_response)
            .await
            .expect("Should fail creating FakeSecResponse with invalid json string as body");
    }
}
