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
//!
//! A daily-index request needs a date before `build()` exists:
//!
//! ```compile_fail,E0599
//! use sec::shared::request::implementations::sec_request::DailyIndex;
//! use sec::shared::request::implementations::sec_request::builder::{DailyIndexBuilder, NoDate};
//!
//! fn build_without_date(builder: DailyIndexBuilder<DailyIndex, NoDate>) {
//!     let _request = builder.build();
//! }
//! ```

use std::marker::PhantomData;

use crate::shared::cik::Cik;
use crate::shared::filing_ref::FilingRef;
use crate::shared::index_date::IndexDate;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::request::SecRequest as SecRequestTrait;
use crate::shared::request::implementations::sec_request::{
    CompanyFacts, DailyIndex, DailyIndexListing, FilingIndex, NotesFile, RequestKind, SecRequest,
    Submissions,
};

/// Typestate marker: no CIK has been set yet.
///
/// While the builder is in this state, [`AllCompanyFactsBuilder::build`] and
/// [`SubmissionsBuilder::build`] are unavailable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoCik;

/// Typestate marker: no Notes period has been set yet.
///
/// While the builder is in this state, [`build`](NotesFileBuilder::build) is unavailable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoPeriod;

/// Typestate marker: no index date has been set yet.
///
/// While the builder is in this state, [`build`](DailyIndexBuilder::build) is unavailable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoDate;

/// Typestate marker: no filing has been set yet.
///
/// While the builder is in this state, [`build`](FilingIndexBuilder::build) is unavailable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoFilingRef;

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

    /// Selects the [`CompanyFacts`] request kind.
    #[must_use]
    pub const fn all_company_facts(self) -> AllCompanyFactsBuilder<NoCik> {
        AllCompanyFactsBuilder { cik: NoCik }
    }

    /// Selects the [`NotesFile`] request kind.
    #[must_use]
    pub const fn notes_file(self) -> NotesFileBuilder<NoPeriod> {
        NotesFileBuilder { period: NoPeriod }
    }

    /// Selects the [`DailyIndex`] request kind.
    #[must_use]
    pub const fn daily_index(self) -> DailyIndexBuilder<DailyIndex, NoDate> {
        DailyIndexBuilder::new()
    }

    /// Selects the [`DailyIndexListing`] request kind.
    #[must_use]
    pub const fn daily_index_listing(self) -> DailyIndexBuilder<DailyIndexListing, NoDate> {
        DailyIndexBuilder::new()
    }

    /// Selects the [`Submissions`] request kind.
    #[must_use]
    pub const fn submissions(self) -> SubmissionsBuilder<NoCik> {
        SubmissionsBuilder { cik: NoCik }
    }

    /// Selects the [`FilingIndex`] request kind.
    #[must_use]
    pub const fn filing_index(self) -> FilingIndexBuilder<NoFilingRef> {
        FilingIndexBuilder {
            filing: NoFilingRef,
        }
    }
}

impl Default for SecRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Typestate builder for the [`FetchAllCompanyFacts`](super::SecRequestType::FetchAllCompanyFacts) request.
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
    pub fn build(self) -> SecRequest<CompanyFacts> {
        SecRequest::new(self.cik)
    }
}

/// Typestate builder for the [`FetchSubmissions`](super::SecRequestType::FetchSubmissions) request.
///
/// The `C` type parameter tracks the CIK. It starts as [`NoCik`]. [`cik`](Self::cik) changes it to
/// [`Cik`]. Only that type has [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionsBuilder<C> {
    cik: C,
}

impl SubmissionsBuilder<NoCik> {
    /// Sets the CIK, unlocking [`build`](SubmissionsBuilder::build).
    #[must_use]
    pub const fn cik(self, cik: Cik) -> SubmissionsBuilder<Cik> {
        SubmissionsBuilder { cik }
    }
}

impl SubmissionsBuilder<Cik> {
    /// Builds the [`SecRequest`]; available only once the CIK is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::cik::Cik;
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    ///
    /// let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
    /// let request = SecRequest::builder().submissions().cik(cik).build();
    ///
    /// let expected_result = "https://data.sec.gov/submissions/CIK0001067983.json";
    ///
    /// let result = request.inner.url().as_str();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn build(self) -> SecRequest<Submissions> {
        SecRequest::new(self.cik)
    }
}

/// Typestate builder for the [`FetchFilingIndex`](super::SecRequestType::FetchFilingIndex) request.
///
/// The `F` type parameter tracks the filing. It starts as [`NoFilingRef`].
/// [`filing_ref`](Self::filing_ref) changes it to [`FilingRef`]. Only that type has
/// [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilingIndexBuilder<F> {
    filing: F,
}

impl FilingIndexBuilder<NoFilingRef> {
    /// Sets the filing, unlocking [`build`](FilingIndexBuilder::build).
    #[must_use]
    pub const fn filing_ref(self, filing: FilingRef) -> FilingIndexBuilder<FilingRef> {
        FilingIndexBuilder { filing }
    }
}

impl FilingIndexBuilder<FilingRef> {
    /// Builds the [`SecRequest`]; available only once the filing is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use sec::shared::cik::Cik;
    /// use sec::shared::filing_ref::FilingRef;
    /// use sec::shared::financial::accession_number::AccessionNumber;
    /// use sec::shared::financial::form::{Form, ParsedForm};
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    ///
    /// let cik = Cik::new("320193").expect("A hardcoded valid CIK should always parse");
    /// let filed_date = NaiveDate::from_ymd_opt(2017, 11, 3)
    ///     .expect("A hardcoded calendar date should always build");
    /// let filing = FilingRef::new(
    ///     cik,
    ///     AccessionNumber::new("0000320193-17-000070"),
    ///     ParsedForm::new(Form::TenK, false),
    ///     filed_date,
    /// );
    /// let request = SecRequest::builder().filing_index().filing_ref(filing).build();
    ///
    /// let expected_result =
    ///     "https://www.sec.gov/Archives/edgar/data/320193/000032019317000070/index.json";
    ///
    /// let result = request.inner.url().as_str();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn build(self) -> SecRequest<FilingIndex> {
        SecRequest::new(self.filing)
    }
}

/// Typestate builder for the [`FetchNotesFile`](super::SecRequestType::FetchNotesFile) request.
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
    pub fn build(self) -> SecRequest<NotesFile> {
        SecRequest::new(self.period)
    }
}

/// Typestate builder for the [`DailyIndex`] and [`DailyIndexListing`] requests.
///
/// `K` is the request kind. The `D` type parameter tracks the date. It starts as [`NoDate`].
/// [`date`](Self::date) changes it to [`IndexDate`]. Only that type has [`build`](Self::build).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyIndexBuilder<K, D> {
    kind: PhantomData<K>,
    date: D,
}

impl<K> DailyIndexBuilder<K, NoDate> {
    const fn new() -> Self {
        Self {
            kind: PhantomData,
            date: NoDate,
        }
    }

    /// Sets the index date, unlocking [`build`](DailyIndexBuilder::build).
    #[must_use]
    pub const fn date(self, date: IndexDate) -> DailyIndexBuilder<K, IndexDate> {
        DailyIndexBuilder {
            kind: PhantomData,
            date,
        }
    }
}

impl<K: RequestKind<Input = IndexDate>> DailyIndexBuilder<K, IndexDate> {
    /// Builds the [`SecRequest`]; available only once the index date is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::index_date::IndexDate;
    /// use sec::shared::request::implementations::sec_request::SecRequest;
    ///
    /// let date = IndexDate::new("2026-10-27").expect("A hardcoded past date should always parse");
    /// let request = SecRequest::builder().daily_index().date(date).build();
    ///
    /// let expected_result = "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261027.idx";
    ///
    /// let result = request.inner.url().as_str();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn build(self) -> SecRequest<K> {
        SecRequest::new(self.date)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use reqwest::{Method, Url};

    use crate::shared::cik::Cik;
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::index_date::IndexDate;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::SecRequest as SecRequestTrait;
    use crate::shared::request::implementations::sec_request::SecRequest;

    fn index_date(date: &str) -> IndexDate {
        IndexDate::new(date).expect(
            "Given a hardcoded past date after 1998-05-18, validation should always succeed",
        )
    }

    #[test]
    fn should_produce_qtr1_url_for_a_daily_index_request_in_march() {
        let date = index_date("2026-03-02");

        let expected_result =
            "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR1/form.20260302.idx";

        let result = SecRequest::builder()
            .daily_index()
            .date(date)
            .build()
            .inner()
            .url()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_qtr2_url_for_a_daily_index_request_in_april() {
        let date = index_date("2026-04-01");

        let expected_result =
            "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR2/form.20260401.idx";

        let result = SecRequest::builder()
            .daily_index()
            .date(date)
            .build()
            .inner()
            .url()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_qtr3_url_for_a_daily_index_request_in_september() {
        let date = index_date("2026-09-30");

        let expected_result =
            "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR3/form.20260930.idx";

        let result = SecRequest::builder()
            .daily_index()
            .date(date)
            .build()
            .inner()
            .url()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_qtr4_url_for_a_daily_index_request_in_october() {
        let date = index_date("2026-10-27");

        let expected_result =
            "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261027.idx";

        let result = SecRequest::builder()
            .daily_index()
            .date(date)
            .build()
            .inner()
            .url()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_the_quarter_listing_url_for_a_daily_index_listing_request() {
        let date = index_date("2026-10-27");

        let expected_result = "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/index.json";

        let result = SecRequest::builder()
            .daily_index_listing()
            .date(date)
            .build()
            .inner()
            .url()
            .to_string();

        assert_eq!(result, expected_result);
    }

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
    fn should_produce_unpadded_cik_and_undashed_accession_number_in_url_for_filing_index_request() {
        let filing = FilingRef::new(
            Cik::new("0000003545")
                .expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047"),
            ParsedForm::new(Form::TenQ, false),
            NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
        );

        let expected_result = Url::parse(
            "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json",
        )
        .expect("Hardcoded URL should always be valid");

        let result = SecRequest::builder()
            .filing_index()
            .filing_ref(filing)
            .build()
            .inner()
            .url()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_filer_cik_not_the_accession_prefix_in_url_for_filing_index_request() {
        let filing = FilingRef::new(
            Cik::new("2113605")
                .expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0001213900-26-112626"),
            ParsedForm::new(Form::TwentyF, false),
            NaiveDate::from_ymd_opt(2026, 10, 27)
                .expect("Given a hardcoded calendar date, the date should always build"),
        );

        let expected_result = Url::parse(
            "https://www.sec.gov/Archives/edgar/data/2113605/000121390026112626/index.json",
        )
        .expect("Hardcoded URL should always be valid");

        let result = SecRequest::builder()
            .filing_index()
            .filing_ref(filing)
            .build()
            .inner()
            .url()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_correct_url_when_building_fetch_submissions_request() {
        let cik = Cik::new("0001234567")
            .expect("Given a hardcoded 10-digit CIK, Cik creation should always succeed");

        let expected_result = Url::parse("https://data.sec.gov/submissions/CIK0001234567.json")
            .expect("Hardcoded URL should always be valid");

        let result = SecRequest::builder()
            .submissions()
            .cik(cik)
            .build()
            .inner()
            .url()
            .clone();

        assert_eq!(result, expected_result);
    }

    fn notes_period(period: &str) -> NotesPeriod {
        NotesPeriod::new(period)
            .expect("Given a published period, NotesPeriod creation should always succeed")
    }

    #[test]
    fn should_produce_quarterly_url_for_notes_file_request() {
        let period = notes_period("2024q1");

        let expected_result = Url::parse(
            "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2024q1_notes.zip",
        )
        .expect("Hardcoded URL should always be valid");

        let result = SecRequest::builder()
            .notes_file()
            .period(period)
            .build()
            .inner()
            .url()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_monthly_url_for_notes_file_request() {
        let period = notes_period("2025_10");

        let expected_result = Url::parse(
            "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2025_10_notes.zip",
        )
        .expect("Hardcoded URL should always be valid");

        let result = SecRequest::builder()
            .notes_file()
            .period(period)
            .build()
            .inner()
            .url()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_use_overridden_file_name_for_2010q1_notes_file_request() {
        let period = notes_period("2010q1");

        let expected_result = Url::parse(
            "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2010q1_notes_1.zip",
        )
        .expect("Hardcoded URL should always be valid");

        let result = SecRequest::builder()
            .notes_file()
            .period(period)
            .build()
            .inner()
            .url()
            .clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_get_method_for_notes_file_request() {
        let period = notes_period("2024q1");

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
