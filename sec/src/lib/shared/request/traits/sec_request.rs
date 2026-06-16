use std::fmt::Debug;

use async_trait::async_trait;

use crate::shared::request::InnerRequest;

/// The domain-level SEC request: builds a transport request from SEC concepts.
///
/// Sits above [`InnerRequest`], translating domain input (such as a CIK and target endpoint) into
/// a ready-to-send transport request. Existing as a trait keeps request construction decoupled
/// from any specific HTTP crate and testable with a fake.
///
/// # Associated Types
///
/// - `Inner`: The underlying transport request, an [`InnerRequest`].
/// - `RequestInput`: The domain input from which the request is built.
#[async_trait]
pub trait SecRequest: Send + Sync + Debug {
    /// The underlying transport request this wraps.
    type Inner: InnerRequest;
    /// The domain input from which the request is built.
    type RequestInput;

    /// Returns a reference to the underlying transport request.
    fn inner(&self) -> &Self::Inner;

    /// Returns the request's HTTP method, delegating to the inner request.
    fn method(&self) -> &<Self::Inner as InnerRequest>::Method {
        self.inner().method()
    }

    /// Returns the request's target URL, delegating to the inner request.
    fn url(&self) -> &<Self::Inner as InnerRequest>::Url {
        self.inner().url()
    }

    /// Builds a request from its domain input.
    fn new(request_input: Self::RequestInput) -> Self;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::shared::cik::Cik;
    use crate::shared::request::traits::SecRequest;

    use crate::tests::fixtures::sample_request::sample_inner_request::fake_inner_request::FakeMethod;
    use crate::tests::fixtures::sample_request::sample_sec_request::{
        FakeRequestInput, FakeSecRequest,
    };

    #[test]
    fn should_create_fake_sec_request_with_expected_url_when_using_constructor() {
        let cik = Cik::new("0000000000").expect("Hardcoded CIK should always be valid");
        let request_input = FakeRequestInput {
            cik: cik.to_string(),
        };

        let expected_result =
            String::from("https://example.com/fetch_all_company_facts/0000000000");

        let result = FakeSecRequest::new(request_input).url().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_fake_sec_request_with_expected_method_when_using_constructor() {
        let cik = Cik::new("0000000000").expect("Hardcoded CIK should always be valid");
        let request_input = FakeRequestInput {
            cik: cik.to_string(),
        };

        let expected_result = FakeMethod::GET;

        let result = FakeSecRequest::new(request_input).method().clone();

        assert_eq!(result, expected_result);
    }
}
