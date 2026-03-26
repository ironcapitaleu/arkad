use crate::shared::request::{InnerRequest, SecRequest, SecRequestType};

use crate::tests::fixtures::sample_request::sample_inner_request::fake_inner_request::{
    FakeInnerRequest, FakeMethod,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FakeSecRequest {
    inner: FakeInnerRequest,
}

impl SecRequest for FakeSecRequest {
    type Inner = FakeInnerRequest;

    fn new(request_type: SecRequestType) -> Self {
        match request_type {
            SecRequestType::FetchAllCompanyFacts { cik } => {
                let method = FakeMethod::GET;
                let url = format!("https://example.com/fetch_all_company_facts/{}", cik);

                let inner_request = FakeInnerRequest::new(method, url);
                Self {
                    inner: inner_request,
                }
            }
        }
    }

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }
}
