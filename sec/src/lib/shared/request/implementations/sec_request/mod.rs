use reqwest::{Method, Request, Url};

use crate::shared::{cik::Cik, request::SecRequest as SecRequestTrait};

pub mod builder;
pub mod constants;

pub use builder::SecRequestBuilder;

use constants::{SEC_COMPANY_FACTS_URL_PREFIX, SEC_COMPANY_FACTS_URL_SUFFIX};

/// A validated SEC API request.
///
/// `SecRequest` wraps a `reqwest::Request` that has been constructed from a
/// high-level [`SecRequestType`], ensuring the correct URL endpoint and HTTP
/// method are used.
#[derive(Debug)]
pub struct SecRequest {
    pub inner: Request,
}

impl serde::Serialize for SecRequest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SecRequest", 2)?;
        state.serialize_field("url", self.inner.url().as_str())?;
        state.serialize_field("method", self.inner.method().as_str())?;
        state.end()
    }
}

impl SecRequest {
    /// Creates a new [`SecRequestBuilder`] for constructing a [`SecRequest`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::cik::Cik;
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    ///
    /// let cik = Cik::new("1067983").expect("Hardcoded CIK should be valid");
    /// let request = SecRequest::builder()
    ///     .all_company_facts()
    ///     .cik(cik)
    ///     .build();
    /// ```
    #[must_use]
    pub const fn builder() -> SecRequestBuilder {
        SecRequestBuilder::new()
    }

    /// Consumes the `SecRequest` and returns the inner `reqwest::Request`.
    #[must_use]
    pub fn into_inner(self) -> Request {
        self.inner
    }

    /// Creates a [`SecRequest`] from a fully-formed [`SecRequestType`].
    pub(crate) fn from_request_type(request_type: SecRequestType) -> Self {
        match request_type {
            SecRequestType::FetchAllCompanyFacts { cik } => {
                let url =
                    format!("{SEC_COMPANY_FACTS_URL_PREFIX}{cik}{SEC_COMPANY_FACTS_URL_SUFFIX}");
                let request = Request::new(
                    Method::GET,
                    Url::parse(&url).expect("Hardcoded URL should always be valid"),
                );
                Self { inner: request }
            }
        }
    }
}

impl SecRequestTrait for SecRequest {
    type Inner = Request;
    type RequestInput = SecRequestType;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn new(request_input: Self::RequestInput) -> Self {
        Self::from_request_type(request_input)
    }
}

impl Clone for SecRequest {
    fn clone(&self) -> Self {
        Self {
            inner: self
                .inner
                .try_clone()
                .expect("SEC requests have no streaming body, so cloning should always succeed"),
        }
    }
}

impl PartialEq for SecRequest {
    fn eq(&self, other: &Self) -> bool {
        self.inner.url() == other.inner.url()
    }
}

impl Eq for SecRequest {}

impl std::hash::Hash for SecRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.url().hash(state);
    }
}

impl PartialOrd for SecRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SecRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.url().cmp(other.inner.url())
    }
}

/// Enum representing the different types of high-level SEC requests that are supported.
///
/// Each variant encodes the input parameters that are required to make the request.
/// Every variant automatically encodes the logic to create a properly formatted request
/// based on the input parameters, that includes setting up the correct URL endpoint and HTTP method for the request.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize)]
#[non_exhaustive]
pub enum SecRequestType {
    /// Fetches all the facts for a given company based on its CIK. This includes all the financial statement data that the company has submitted to the SEC over the years, such as balance sheets, income statements, cash flow statements, and other relevant financial information.
    FetchAllCompanyFacts { cik: Cik },
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use reqwest::{Method, Url};

    use super::SecRequest;
    use crate::shared::{cik::Cik, request::SecRequest as SecRequestTrait};

    #[test]
    fn should_automatically_create_inner_request_method_when_specifying_sec_request_type() {
        let cik = Cik::new("0001234567").expect("Hardcoded CIK should be valid");

        let expected_result = Method::GET;

        let result = SecRequest::builder()
            .all_company_facts()
            .cik(cik)
            .build()
            .inner()
            .method()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_automatically_create_inner_request_url_when_specifying_sec_request_type() {
        let cik = Cik::new("0001234567").expect("Hardcoded CIK should be valid");

        let expected_result =
            Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001234567.json")
                .expect("Hardcoded URL should always be valid");

        let result = SecRequest::builder()
            .all_company_facts()
            .cik(cik)
            .build()
            .inner()
            .url()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_url_and_method_when_serialized_to_json() {
        let cik = Cik::new("0001234567").expect("Hardcoded CIK should be valid");
        let request = SecRequest::builder().all_company_facts().cik(cik).build();

        let expected_result = serde_json::json!({
            "url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0001234567.json",
            "method": "GET",
        });

        let result = serde_json::to_value(&request).expect("SecRequest should serialize to JSON");

        assert_eq!(result, expected_result);
    }

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_unpin<T: Unpin>() {}

    #[test]
    fn should_be_send() {
        assert_send::<SecRequest>();
    }

    #[test]
    fn should_be_sync() {
        assert_sync::<SecRequest>();
    }

    #[test]
    fn should_be_unpin() {
        assert_unpin::<SecRequest>();
    }
}
