use std::fmt::Debug;

/// A trait defining how a request has to be created. This is used to decouple from third party libraries.
pub trait InnerRequest: Send + Sync + Debug {
    /// This type represents the HTTP method that the client is going to execute against the URL.
    type Method;
    /// This type represents the endpoint that the client is going to execute the request method against.
    type Url;

    /// Creates a new request for the client.
    /// Returns an instance of a Request struct.
    fn new(method: Self::Method, url: Self::Url) -> Self;
    fn method(&self) -> &Self::Method;
    fn url(&self) -> &Self::Url;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::shared::request::traits::inner::InnerRequest;
    use crate::tests::fixtures::sample_request::sample_inner_request::fake_inner_request::{
        FakeInnerRequest, FakeMethod,
    };

    #[test]
    fn should_create_fake_request_when_using_constructor() {
        let method = FakeMethod::GET;
        let url = String::from("https://example.com");

        let expected_result = FakeInnerRequest {
            method: FakeMethod::GET,
            url: String::from("https://example.com"),
        };

        let result = FakeInnerRequest::new(method, url);

        assert_eq!(result.method(), expected_result.method());
    }
}
