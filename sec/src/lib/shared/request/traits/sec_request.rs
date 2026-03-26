use std::fmt::Debug;

use async_trait::async_trait;

use crate::shared::{cik::Cik, request::InnerRequest};

/// A trait defining the behavior a high-level `SecRequest` is expected to implement. This is used to make requests based on high-level domain knowledge to the SEC endpoints.
#[async_trait]
pub trait SecRequest: Send + Sync + Debug {
    type Inner: InnerRequest;

    fn inner(&self) -> &Self::Inner;

    fn method(&self) -> &<Self::Inner as InnerRequest>::Method {
        self.inner().method()
    }

    fn url(&self) -> &<Self::Inner as InnerRequest>::Url {
        self.inner().url()
    }

    fn new(request_type: SecRequestType) -> Self;
}

/// Enum representing the different types of high-level SEC requests that are supported.
///
/// Each variant encodes the input parameters that are required to make the request.
/// Every variant automatically encodes the logic to create a properly formatted request
/// based on the input parameters, that includes setting up the correct URL endpoint and HTTP method for the request.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SecRequestType {
    /// Fetches all the facts for a given company based on its CIK. This includes all the financial statement data that the company has submitted to the SEC over the years, such as balance sheets, income statements, cash flow statements, and other relevant financial information.
    FetchAllCompanyFacts { cik: Cik },
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::shared::cik::Cik;
    use crate::shared::request::SecRequestType;
    use crate::shared::request::traits::SecRequest;

    use crate::tests::fixtures::sample_request::sample_inner_request::fake_inner_request::FakeMethod;
    use crate::tests::fixtures::sample_request::sample_sec_request::FakeSecRequest;

    #[test]
    fn should_create_fake_sec_request_with_expected_url_when_using_constructor() {
        let cik = Cik::new("0000000000").unwrap();

        let expected_result =
            String::from("https://example.com/fetch_all_company_facts/0000000000");

        let result = FakeSecRequest::new(SecRequestType::FetchAllCompanyFacts { cik })
            .url()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_fake_sec_request_with_expected_method_when_using_constructor() {
        let cik = Cik::new("0000000000").unwrap();

        let expected_result = FakeMethod::GET;

        let result = FakeSecRequest::new(SecRequestType::FetchAllCompanyFacts { cik })
            .method()
            .clone();

        assert_eq!(result, expected_result);
    }
}
