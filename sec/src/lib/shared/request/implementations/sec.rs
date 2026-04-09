use reqwest::{Method, Request, Url};

use crate::shared::{cik::Cik, request::SecRequest as SecRequestTrait};

#[derive(Debug)]
pub struct SecRequest {
    inner: Request,
}

impl SecRequestTrait for SecRequest {
    type Inner = Request;
    type RequestInput = SecRequestType;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn new(request_input: Self::RequestInput) -> Self {
        match request_input {
            SecRequestType::FetchAllCompanyFacts { cik } => {
                let url = format!("https://data.sec.gov/api/xbrl/companyfacts/{cik}.json");
                let request = Request::new(
                    Method::GET,
                    Url::parse(&url).expect("Hardcoded URL should always be valid"),
                );
                Self { inner: request }
            }
        }
    }
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

impl SecRequestType {
    #[must_use]
    pub const fn new_fetch_all_company_facts(cik: Cik) -> Self {
        Self::FetchAllCompanyFacts { cik }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use reqwest::{Method, Url};

    use super::{SecRequest, SecRequestType};
    use crate::shared::{cik::Cik, request::SecRequest as SecRequestTrait};

    #[test]
    fn should_automatically_create_inner_request_method_when_specifying_sec_request_type() {
        let cik = Cik::new("0001234567").expect("Hardcoded CIK should be valid");
        let sec_request_type = SecRequestType::new_fetch_all_company_facts(cik);

        let expected_result = Method::GET;

        let result = SecRequest::new(sec_request_type).inner().method().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_automatically_create_inner_request_url_when_specifying_sec_request_type() {
        let cik = Cik::new("0001234567").expect("Hardcoded CIK should be valid");
        let sec_request_type = SecRequestType::new_fetch_all_company_facts(cik);

        let expected_result =
            Url::parse("https://data.sec.gov/api/xbrl/companyfacts/0001234567.json")
                .expect("Hardcoded URL should always be valid");

        let result = SecRequest::new(sec_request_type).inner().url().clone();

        assert_eq!(result, expected_result);
    }
}
