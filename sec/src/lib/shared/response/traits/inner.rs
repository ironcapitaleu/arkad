use std::fmt::Debug;

/// A trait defining the interface of an HTTP response. This is used to decouple from third party libraries.
pub trait InnerResponse: Send + Sync + Debug {
    /// This type represents the HTTP method that the client used to execute the corresponding HTTP request.
    type Method;

    /// This type represents the endpoint that the client executed the corresponding HTTP request against.
    type Url;

    /// This type represents the headers of the HTTP response.
    type Header;

    /// This type represents the body of the HTTP response.
    type Body;

    /// This type represents the status code of the HTTP response.
    type StatusCode;

    /// This type represents the content type of the HTTP response.
    type ContentType;

    // TODO: check if needed, if not delete --- IGNORE ---
    type Error;

    fn method(&self) -> &Self::Method;

    fn url(&self) -> &Self::Url;

    fn headers(&self) -> &Self::Header;

    fn body(&self) -> &Self::Body;

    fn status_code(&self) -> &Self::StatusCode;

    fn content_type(&self) -> &Self::ContentType;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::shared::response::InnerResponse;
    use crate::tests::fixtures::sample_response::sample_inner_response::{
        FakeInnerResponse, FakeMethod,
    };

    #[test]
    fn should_create_fake_response_with_specified_method_when_using_constructor() {
        let method = FakeMethod::GET;
        let url = String::from("https://example.com");
        let body = String::from("This is a fake response body.");
        let header = String::from("This is a fake response header.");
        let status_code = 200;
        let content_type = String::from("application/json");

        let expected_result = FakeInnerResponse {
            method: FakeMethod::GET,
            url: String::from("https://example.com"),
            body: String::from("This is a fake response body."),
            header: String::from("This is a fake response header."),
            status_code: 200,
            content_type: String::from("application/json"),
        }
        .method()
        .clone();

        let result = FakeInnerResponse::new(method, url, body, header, status_code, content_type)
            .method()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_fake_response_with_specified_url_when_using_constructor() {
        let method = FakeMethod::GET;
        let url = String::from("https://example.com");
        let body = String::from("This is a fake response body.");
        let header = String::from("This is a fake response header.");
        let status_code = 200;
        let content_type = String::from("application/json");

        let expected_result = FakeInnerResponse {
            method: FakeMethod::GET,
            url: String::from("https://example.com"),
            body: String::from("This is a fake response body."),
            header: String::from("This is a fake response header."),
            status_code: 200,
            content_type: String::from("application/json"),
        }
        .url()
        .clone();

        let result = FakeInnerResponse::new(method, url, body, header, status_code, content_type)
            .url()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_fake_response_with_specified_body_when_using_constructor() {
        let method = FakeMethod::GET;
        let url = String::from("https://example.com");
        let body = String::from("This is a fake response body.");
        let header = String::from("This is a fake response header.");
        let status_code = 200;
        let content_type = String::from("application/json");

        let expected_result = FakeInnerResponse {
            method: FakeMethod::GET,
            url: String::from("https://example.com"),
            body: String::from("This is a fake response body."),
            header: String::from("This is a fake response header."),
            status_code: 200,
            content_type: String::from("application/json"),
        }
        .body()
        .clone();

        let result = FakeInnerResponse::new(method, url, body, header, status_code, content_type)
            .body()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_fake_response_with_specified_status_code_when_using_constructor() {
        let method = FakeMethod::GET;
        let url = String::from("https://example.com");
        let body = String::from("This is a fake response body.");
        let header = String::from("This is a fake response header.");
        let status_code = 200;
        let content_type = String::from("application/json");

        let expected_result = FakeInnerResponse {
            method: FakeMethod::GET,
            url: String::from("https://example.com"),
            body: String::from("This is a fake response body."),
            header: String::from("This is a fake response header."),
            status_code: 200,
            content_type: String::from("application/json"),
        }
        .status_code()
        .clone();

        let result = FakeInnerResponse::new(method, url, body, header, status_code, content_type)
            .status_code()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_fake_response_with_specified_content_type_when_using_constructor() {
        let method = FakeMethod::GET;
        let url = String::from("https://example.com");
        let body = String::from("This is a fake response body.");
        let header = String::from("This is a fake response header.");
        let status_code = 200;
        let content_type = String::from("application/json");

        let expected_result = FakeInnerResponse {
            method: FakeMethod::GET,
            url: String::from("https://example.com"),
            body: String::from("This is a fake response body."),
            header: String::from("This is a fake response header."),
            status_code: 200,
            content_type: String::from("application/json"),
        }
        .content_type()
        .clone();

        let result = FakeInnerResponse::new(method, url, body, header, status_code, content_type)
            .content_type()
            .clone();

        assert_eq!(result, expected_result);
    }
}
