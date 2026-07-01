use std::fmt::Debug;

/// The transport-level request: a method and a URL, decoupled from any HTTP crate.
///
/// Abstracts over a concrete request type so the domain layer doesn't depend on a specific
/// transport. Implemented for the `reqwest` request type and the test fakes;
/// [`SecRequest`](super::SecRequest) builds on top of it.
///
/// # Associated Types
///
/// Each implementor binds these to its transport's concrete types, which is what lets the domain
/// layer stay independent of any specific HTTP crate:
///
/// - `Method`: The type representing the request's HTTP method.
/// - `Url`: The type representing the request's URL.
pub trait InnerRequest: Send + Sync + Debug {
    /// The type representing the request's HTTP method.
    type Method;
    /// The type representing the request's URL.
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
