//! Typestate builder for constructing [`SecRequest`] instances with compile-time safety.
//!
//! This module provides [`SecRequestBuilder`] and associated marker types that enable
//! type-safe construction of [`SecRequest`]. The builder uses a consuming
//! typestate pattern to ensure all required fields are set before construction, and that the
//! correct fields are available based on the selected request type.
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

use std::marker::PhantomData;

use crate::shared::cik::Cik;
use crate::shared::request::implementations::sec_request::{SecRequest, SecRequestType};

/// Marker type indicating no request type has been selected yet.
///
/// When this marker is set, only request type selectors (e.g.,
/// [`all_company_facts()`](SecRequestBuilder::all_company_facts)) are available on the builder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoRequestType;

/// Marker type indicating the [`FetchAllCompanyFacts`](SecRequestType::FetchAllCompanyFacts)
/// request type has been selected.
///
/// When this marker is set via [`all_company_facts()`](SecRequestBuilder::all_company_facts),
/// the builder enables field setters specific to this request type, such as
/// [`cik()`](SecRequestBuilder::cik).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllCompanyFacts;

/// Marker type indicating no CIK has been provided yet.
///
/// This marker prevents the builder from calling [`build()`](SecRequestBuilder::build)
/// until [`cik()`](SecRequestBuilder::cik) is called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoCik;

/// Typestate builder for constructing [`SecRequest`] instances with compile-time safety.
///
/// This builder uses a consuming typestate pattern where generic parameters track which
/// fields have been set. Methods are only available when the builder is in the correct
/// state, enforced at compile time through impl block constraints.
///
/// # Type Parameters
///
/// * `RT` — The request type marker. Starts as [`NoRequestType`] and transitions to a
///   request-type-specific marker (e.g., [`AllCompanyFacts`]) when a request type is selected.
/// * `C` — The CIK field marker. Starts as [`NoCik`] and transitions to [`Cik`] when
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
pub struct SecRequestBuilder<RT, C> {
    _request_type: PhantomData<RT>,
    cik: C,
}

impl SecRequestBuilder<NoRequestType, NoCik> {
    /// Creates a new [`SecRequestBuilder`] with no fields set.
    ///
    /// # Returns
    ///
    /// A new [`SecRequestBuilder`] instance in its initial state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _request_type: PhantomData,
            cik: NoCik,
        }
    }

    /// Selects the [`FetchAllCompanyFacts`](SecRequestType::FetchAllCompanyFacts) request type.
    ///
    /// Consumes the builder and returns a new builder configured for the
    /// `FetchAllCompanyFacts` request type, enabling the [`cik()`](SecRequestBuilder::cik) method.
    ///
    /// # Returns
    ///
    /// A new [`SecRequestBuilder`] instance with the [`AllCompanyFacts`] marker set.
    #[must_use]
    pub const fn all_company_facts(self) -> SecRequestBuilder<AllCompanyFacts, NoCik> {
        SecRequestBuilder {
            _request_type: PhantomData,
            cik: NoCik,
        }
    }
}

impl SecRequestBuilder<AllCompanyFacts, NoCik> {
    /// Sets the CIK for the request.
    ///
    /// This method is only available when the [`AllCompanyFacts`] request type has been
    /// selected and no CIK has been provided yet.
    ///
    /// # Arguments
    ///
    /// * `cik` — The validated [`Cik`] to use for the request.
    ///
    /// # Returns
    ///
    /// A new [`SecRequestBuilder`] instance with the CIK field set, enabling
    /// [`build()`](SecRequestBuilder::build).
    #[must_use]
    pub const fn cik(self, cik: Cik) -> SecRequestBuilder<AllCompanyFacts, Cik> {
        SecRequestBuilder {
            _request_type: PhantomData,
            cik,
        }
    }
}

impl SecRequestBuilder<AllCompanyFacts, Cik> {
    /// Builds the [`SecRequest`] from the fully configured builder state.
    ///
    /// This method is only available when:
    /// - The request type has been set to [`AllCompanyFacts`] via
    ///   [`all_company_facts()`](SecRequestBuilder::all_company_facts)
    /// - The CIK has been provided via [`cik()`](SecRequestBuilder::cik)
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

impl Default for SecRequestBuilder<NoRequestType, NoCik> {
    fn default() -> Self {
        Self::new()
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
        assert_send::<super::SecRequestBuilder<super::NoRequestType, super::NoCik>>();
    }

    #[test]
    fn should_be_sync_when_in_initial_state() {
        assert_sync::<super::SecRequestBuilder<super::NoRequestType, super::NoCik>>();
    }

    #[test]
    fn should_be_send_when_in_all_company_facts_state() {
        assert_send::<super::SecRequestBuilder<super::AllCompanyFacts, super::NoCik>>();
    }

    #[test]
    fn should_be_sync_when_in_all_company_facts_state() {
        assert_sync::<super::SecRequestBuilder<super::AllCompanyFacts, super::NoCik>>();
    }

    #[test]
    fn should_be_send_when_fully_configured() {
        assert_send::<super::SecRequestBuilder<super::AllCompanyFacts, Cik>>();
    }

    #[test]
    fn should_be_sync_when_fully_configured() {
        assert_sync::<super::SecRequestBuilder<super::AllCompanyFacts, Cik>>();
    }
}
