use crate::shared::request::{InnerRequest, SecRequest};

use crate::tests::fixtures::sample_request::sample_inner_request::fake_inner_request::{
    FakeInnerRequest, FakeMethod,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FakeSecRequest {
    inner: FakeInnerRequest,
}

impl SecRequest for FakeSecRequest {
    type Inner = FakeInnerRequest;
    type RequestInput = FakeRequestInput;

    fn new(request_input: Self::RequestInput) -> Self {
        let method = FakeMethod::GET;
        let url = format!(
            "https://example.com/fetch_all_company_facts/{}",
            request_input.cik
        );

        let inner_request = FakeInnerRequest::new(method, url);
        Self {
            inner: inner_request,
        }
    }

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }
}
pub struct FakeRequestInput {
    pub cik: String,
}
