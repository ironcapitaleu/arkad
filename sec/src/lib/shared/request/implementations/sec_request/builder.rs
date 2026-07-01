//! # SEC Request Builder
//!
//! Provides the typestate [`SecRequestBuilder`] for constructing a [`SecRequest`] with
//! compile-time guarantees.
//!
//! Selecting a request kind transitions into a variant-specific builder (e.g.
//! [`AllCompanyFactsBuilder`]) that carries only that variant's fields, and `build()` only exists
//! once every required field is set. This encodes the request's required inputs in the type
//! system, so a missing or irrelevant field is a compile error rather than a runtime check.
//!
//! ## Usage
//!
//! ```
//! use sec::shared::cik::Cik;
//! use sec::shared::request::implementations::sec_request::SecRequest;
//!
//! let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
//! let request = SecRequest::builder().all_company_facts().cik(cik).build();
//! ```

use crate::shared::cik::Cik;
use crate::shared::request::implementations::sec_request::{SecRequest, SecRequestType};

/// Typestate marker: no CIK has been set yet.
///
/// While the builder is in this state, [`build`](AllCompanyFactsBuilder::build) is unavailable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoCik;

/// Entry point for building a [`SecRequest`].
///
/// Dispatches to a variant-specific builder via a request-kind selector such as
/// [`all_company_facts`](Self::all_company_facts).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecRequestBuilder;

impl SecRequestBuilder {
    /// Creates a new builder ready for request-kind selection.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Selects the [`FetchAllCompanyFacts`](SecRequestType::FetchAllCompanyFacts) request kind.
    #[must_use]
    pub const fn all_company_facts(self) -> AllCompanyFactsBuilder<NoCik> {
        AllCompanyFactsBuilder { cik: NoCik }
    }
}

impl Default for SecRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Typestate builder for the [`FetchAllCompanyFacts`](SecRequestType::FetchAllCompanyFacts) request.
///
/// The `C` type parameter tracks the CIK field's state: it starts as [`NoCik`] and becomes [`Cik`]
/// once [`cik`](Self::cik) is called, which is what gates [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllCompanyFactsBuilder<C> {
    cik: C,
}

impl AllCompanyFactsBuilder<NoCik> {
    /// Sets the CIK, unlocking [`build`](AllCompanyFactsBuilder::build).
    #[must_use]
    pub const fn cik(self, cik: Cik) -> AllCompanyFactsBuilder<Cik> {
        AllCompanyFactsBuilder { cik }
    }
}

impl AllCompanyFactsBuilder<Cik> {
    /// Builds the [`SecRequest`]; available only once the CIK is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::cik::Cik;
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    ///
    /// let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
    /// let request = SecRequest::builder().all_company_facts().cik(cik).build();
    /// ```
    #[must_use]
    pub fn build(self) -> SecRequest {
        let request_type = SecRequestType::FetchAllCompanyFacts { cik: self.cik };
        SecRequest::from_request_type(request_type)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use reqwest::{Method, Url};

    use crate::shared::cik::Cik;
    use crate::shared::request::SecRequest as SecRequestTrait;
    use crate::shared::request::implementations::sec_request::SecRequest;

    #[test]
    fn should_produce_correct_url_when_building_fetch_all_company_facts_request() {
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
    fn should_produce_get_method_when_building_fetch_all_company_facts_request() {
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
    fn should_create_default_builder_when_new_is_used_with_no_fields_set() {
        let expected_result = SecRequest::builder();

        let result = super::SecRequestBuilder::new();

        assert_eq!(result, expected_result);
    }

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn should_be_send_when_in_initial_state() {
        assert_send::<super::SecRequestBuilder>();
    }

    #[test]
    fn should_be_sync_when_in_initial_state() {
        assert_sync::<super::SecRequestBuilder>();
    }

    #[test]
    fn should_be_send_when_in_all_company_facts_state_without_cik() {
        assert_send::<super::AllCompanyFactsBuilder<super::NoCik>>();
    }

    #[test]
    fn should_be_sync_when_in_all_company_facts_state_without_cik() {
        assert_sync::<super::AllCompanyFactsBuilder<super::NoCik>>();
    }

    #[test]
    fn should_be_send_when_in_all_company_facts_state_with_cik() {
        assert_send::<super::AllCompanyFactsBuilder<Cik>>();
    }

    #[test]
    fn should_be_sync_when_in_all_company_facts_state_with_cik() {
        assert_sync::<super::AllCompanyFactsBuilder<Cik>>();
    }
}
