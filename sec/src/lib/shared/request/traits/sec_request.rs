use std::fmt::Debug;

use async_trait::async_trait;

use crate::shared::request::InnerRequest;

/// A trait defining the behavior a high-level `SecRequest` is expected to implement. This is used to make requests based on high-level domain knowledge to the SEC endpoints.
#[async_trait]
pub trait SecRequest: Send + Sync + Debug {
    type Inner: InnerRequest;
    type RequestInput;

    fn inner(&self) -> &Self::Inner;

    fn method(&self) -> &<Self::Inner as InnerRequest>::Method {
        self.inner().method()
    }

    fn url(&self) -> &<Self::Inner as InnerRequest>::Url {
        self.inner().url()
    }

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
        let cik = Cik::new("0000000000").unwrap();
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
        let cik = Cik::new("0000000000").unwrap();
        let request_input = FakeRequestInput {
            cik: cik.to_string(),
        };

        let expected_result = FakeMethod::GET;

        let result = FakeSecRequest::new(request_input).method().clone();

        assert_eq!(result, expected_result);
    }
}
