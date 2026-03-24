use crate::shared::request::traits::inner::InnerRequest;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FakeInnerRequest {
    pub method: FakeMethod,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FakeMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl InnerRequest for FakeInnerRequest {
    type Method = FakeMethod;
    type Url = String;

    fn new(method: Self::Method, url: Self::Url) -> Self {
        Self { method, url }
    }
}
