use std::fmt::Debug;

/// A raw HTTP request: a method and a URL, decoupled from any specific HTTP crate.
///
/// Abstracts over concrete request types so the domain layer doesn't depend on a specific
/// HTTP library.
///
/// # Associated Types
///
/// - `Method`: The request's HTTP method.
/// - `Url`: The request's URL.
pub trait InnerRequest: Send + Sync + Debug {
    /// The request's HTTP method.
    type Method;
    /// The request's URL.
    type Url;

    /// Builds a request from a method and a URL.
    fn new(method: Self::Method, url: Self::Url) -> Self;

    /// Returns a reference to the request's HTTP method.
    fn method(&self) -> &Self::Method;

    /// Returns a reference to the request's target URL.
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
        }
        .method()
        .clone();

        let result = FakeInnerRequest::new(method, url).method().clone();

        assert_eq!(result, expected_result);
    }
}
