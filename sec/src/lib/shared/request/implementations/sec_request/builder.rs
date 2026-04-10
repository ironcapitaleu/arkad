//! Typestate builder for constructing [`SecRequest`] instances with compile-time safety.
//!
//! This module provides [`SecRequestBuilder`] as the entry point and variant-specific
//! builders (e.g., [`AllCompanyFactsBuilder`]) that enable type-safe construction of
//! [`SecRequest`]. Each request type variant gets its own builder struct carrying only
//! the fields that variant requires.
//!
//! # Type Safety
//!
//! The builder prevents invalid states through compile-time type checking:
//! - Cannot call `build()` without selecting a request type and providing all required inputs
//! - Cannot set fields that are not relevant to the selected request type
//! - Cannot set the same field twice
//!
//! # Examples
//!
//! ```
//! use sec::shared::cik::Cik;
//! use sec::shared::request::implementations::sec_request::SecRequest;
//!
//! let cik = Cik::new("1067983").expect("Hardcoded CIK should be valid");
//! let request = SecRequest::builder()
//!     .all_company_facts()
//!     .cik(cik)
//!     .build();
//! ```

use crate::shared::cik::Cik;
use crate::shared::request::implementations::sec_request::{SecRequest, SecRequestType};

/// Marker type indicating no CIK has been provided yet.
///
/// This marker prevents [`AllCompanyFactsBuilder`] from calling
/// [`build()`](AllCompanyFactsBuilder::build) until
/// [`cik()`](AllCompanyFactsBuilder::cik) is called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoCik;

/// Entry-point builder for constructing a [`SecRequest`].
///
/// This builder serves as a dispatcher — call a request type selector method
/// (e.g., [`all_company_facts()`](Self::all_company_facts)) to transition into
/// a variant-specific builder that carries only the fields that variant requires.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecRequestBuilder;

impl SecRequestBuilder {
    /// Creates a new [`SecRequestBuilder`].
    ///
    /// # Returns
    ///
    /// A new [`SecRequestBuilder`] instance ready for request type selection.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Selects the [`FetchAllCompanyFacts`](SecRequestType::FetchAllCompanyFacts) request type.
    ///
    /// Consumes the builder and returns an [`AllCompanyFactsBuilder`] awaiting a CIK.
    ///
    /// # Returns
    ///
    /// A new [`AllCompanyFactsBuilder`] instance with no fields set.
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

/// Typestate builder for the [`FetchAllCompanyFacts`](SecRequestType::FetchAllCompanyFacts)
/// request variant.
///
/// This builder is returned by [`SecRequestBuilder::all_company_facts()`] and uses a
/// consuming typestate pattern to ensure the required CIK is provided before building.
///
/// # Type Parameters
///
/// * `C` — The CIK field state. Starts as [`NoCik`] and transitions to [`Cik`] when
///   [`cik()`](Self::cik) is called.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllCompanyFactsBuilder<C> {
    cik: C,
}

impl AllCompanyFactsBuilder<NoCik> {
    /// Sets the CIK for the request.
    ///
    /// # Arguments
    ///
    /// * `cik` — The validated [`Cik`] to use for the request.
    ///
    /// # Returns
    ///
    /// A new [`AllCompanyFactsBuilder`] instance with the CIK field set, enabling
    /// [`build()`](AllCompanyFactsBuilder::build).
    #[must_use]
    pub const fn cik(self, cik: Cik) -> AllCompanyFactsBuilder<Cik> {
        AllCompanyFactsBuilder { cik }
    }
}

impl AllCompanyFactsBuilder<Cik> {
    /// Builds the [`SecRequest`] from the fully configured builder state.
    ///
    /// This method is only available when the CIK has been provided via
    /// [`cik()`](AllCompanyFactsBuilder::cik).
    ///
    /// # Returns
    ///
    /// A fully configured [`SecRequest`] instance ready for execution.
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
