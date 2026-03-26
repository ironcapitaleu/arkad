use std::fmt::Debug;

use async_trait::async_trait;

use crate::shared::{cik::Cik, request::InnerRequest};

/// A trait defining the behavior a high-level `SecRequest` is expected to implement. This is used to make requests based on high-level domain knowledge to the SEC endpoints.
#[async_trait]
pub trait SecRequest: Send + Sync + Debug {
    type Inner: InnerRequest;

    fn inner(&self) -> &Self::Inner;

    fn new(request_type: SecRequestType) -> Self::Inner;
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
