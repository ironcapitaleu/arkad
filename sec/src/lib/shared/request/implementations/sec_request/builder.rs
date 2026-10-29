//! # SEC Request Builder
//!
//! Provides the typestate [`SecRequestBuilder`] for constructing an [`SecRequest`] with
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
//!
//! let expected_result = "https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json";
//!
//! let result = request.inner.url().as_str();
//!
//! assert_eq!(result, expected_result);
//! ```
//!
//! A Notes file request needs a period before `build()` exists:
//!
//! ```compile_fail,E0599
//! use sec::shared::request::implementations::sec_request::builder::{NoPeriod, NotesFileBuilder};
//!
//! fn build_without_period(builder: NotesFileBuilder<NoPeriod>) {
//!     let _request = builder.build();
//! }
//! ```

use crate::shared::cik::Cik;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::request::implementations::sec_request::{SecRequest, SecRequestType};

/// Typestate marker: no CIK has been set yet.
///
/// While the builder is in this state, [`build`](AllCompanyFactsBuilder::build) is unavailable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoCik;

/// Typestate marker: no Notes period has been set yet.
///
/// While the builder is in this state, [`build`](NotesFileBuilder::build) is unavailable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoPeriod;

/// Entry point for building an [`SecRequest`].
///
/// Dispatches to a variant-specific builder via a request-kind selector such as
/// [`all_company_facts`](Self::all_company_facts).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecRequestBuilder;

impl SecRequestBuilder {
    /// Creates a new [`SecRequestBuilder`].
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Selects the [`FetchAllCompanyFacts`](SecRequestType::FetchAllCompanyFacts) request kind.
    #[must_use]
    pub const fn all_company_facts(self) -> AllCompanyFactsBuilder<NoCik> {
        AllCompanyFactsBuilder { cik: NoCik }
    }

    /// Selects the [`FetchNotesFile`](SecRequestType::FetchNotesFile) request kind.
    #[must_use]
    pub const fn notes_file(self) -> NotesFileBuilder<NoPeriod> {
        NotesFileBuilder { period: NoPeriod }
    }
}

impl Default for SecRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Typestate builder for the [`FetchAllCompanyFacts`](SecRequestType::FetchAllCompanyFacts) request.
///
/// The `C` type parameter tracks the CIK. It starts as [`NoCik`]. [`cik`](Self::cik) changes it to
/// [`Cik`]. Only that type has [`build`](Self::build).
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
    ///
    /// let expected_result = "https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json";
    ///
    /// let result = request.inner.url().as_str();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn build(self) -> SecRequest {
        let request_type = SecRequestType::FetchAllCompanyFacts { cik: self.cik };
        SecRequest::from_request_type(request_type)
    }
}

/// Typestate builder for the [`FetchNotesFile`](SecRequestType::FetchNotesFile) request.
///
/// The `P` type parameter tracks the period. It starts as [`NoPeriod`]. [`period`](Self::period)
/// changes it to [`NotesPeriod`]. Only that type has [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotesFileBuilder<P> {
    period: P,
}

impl NotesFileBuilder<NoPeriod> {
    /// Sets the Notes period, unlocking [`build`](NotesFileBuilder::build).
    #[must_use]
    pub const fn period(self, period: NotesPeriod) -> NotesFileBuilder<NotesPeriod> {
        NotesFileBuilder { period }
    }
}

impl NotesFileBuilder<NotesPeriod> {
    /// Builds the [`SecRequest`]; available only once the Notes period is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::notes_period::NotesPeriod;
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    ///
    /// let period = NotesPeriod::new("2024q1").expect("A hardcoded valid period should always parse");
    /// let request = SecRequest::builder().notes_file().period(period).build();
    ///
    /// let expected_result = "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2024q1_notes.zip";
    ///
    /// let result = request.inner.url().as_str();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn build(self) -> SecRequest {
        let request_type = SecRequestType::FetchNotesFile {
            period: self.period,
        };
        SecRequest::from_request_type(request_type)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use reqwest::{Method, Url};

    use crate::shared::cik::Cik;
    use crate::shared::notes_period::NotesPeriod;
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

    fn build_notes_file_url(period: &str) -> Url {
        let period = NotesPeriod::new(period)
            .expect("Given a published period, NotesPeriod creation should always succeed");
        SecRequest::builder()
            .notes_file()
            .period(period)
            .build()
            .inner()
            .url()
            .clone()
    }

    #[test]
    fn should_produce_quarterly_url_for_notes_file_request() {
        let expected_result = Url::parse(
            "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2024q1_notes.zip",
        )
        .expect("Hardcoded URL should always be valid");

        let result = build_notes_file_url("2024q1");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_monthly_url_for_notes_file_request() {
        let expected_result = Url::parse(
            "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2025_10_notes.zip",
        )
        .expect("Hardcoded URL should always be valid");

        let result = build_notes_file_url("2025_10");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_use_overridden_file_name_for_2010q1_notes_file_request() {
        let expected_result = Url::parse(
            "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2010q1_notes_1.zip",
        )
        .expect("Hardcoded URL should always be valid");

        let result = build_notes_file_url("2010q1");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_get_method_for_notes_file_request() {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a published period, NotesPeriod creation should always succeed");

        let expected_result = Method::GET;

        let result = SecRequest::builder()
            .notes_file()
            .period(period)
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

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_be_send_for_sec_request_builder() {
        implements_send::<super::SecRequestBuilder>();
    }

    #[test]
    const fn should_be_sync_for_sec_request_builder() {
        implements_sync::<super::SecRequestBuilder>();
    }

    #[test]
    const fn should_be_send_for_all_company_facts_builder_without_cik() {
        implements_send::<super::AllCompanyFactsBuilder<super::NoCik>>();
    }

    #[test]
    const fn should_be_sync_for_all_company_facts_builder_without_cik() {
        implements_sync::<super::AllCompanyFactsBuilder<super::NoCik>>();
    }

    #[test]
    const fn should_be_send_for_all_company_facts_builder_with_cik() {
        implements_send::<super::AllCompanyFactsBuilder<Cik>>();
    }

    #[test]
    const fn should_be_sync_for_all_company_facts_builder_with_cik() {
        implements_sync::<super::AllCompanyFactsBuilder<Cik>>();
    }

    #[test]
    const fn should_be_send_for_notes_file_builder_without_period() {
        implements_send::<super::NotesFileBuilder<super::NoPeriod>>();
    }

    #[test]
    const fn should_be_sync_for_notes_file_builder_without_period() {
        implements_sync::<super::NotesFileBuilder<super::NoPeriod>>();
    }

    #[test]
    const fn should_be_send_for_notes_file_builder_with_period() {
        implements_send::<super::NotesFileBuilder<NotesPeriod>>();
    }

    #[test]
    const fn should_be_sync_for_notes_file_builder_with_period() {
        implements_sync::<super::NotesFileBuilder<NotesPeriod>>();
    }
}
