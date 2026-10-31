//! # SEC Request
//!
//! Provides the [`SecRequest`]. Given a request kind and its domain input (e.g. a CIK or a Notes period),
//! it assembles a fully-formed HTTP request — endpoint, method, and all — ready to dispatch.
//! Built through an [`SecRequestBuilder`], or through
//! [`SecRequest::new`](crate::shared::request::SecRequest::new) in code that is generic over the kind.
//!
//! ## Modules
//!
//! - [`builder`]: The [`SecRequestBuilder`] for constructing requests fluently.
//! - [`constants`]: The endpoint URL fragments.
//! - [`kind`]: The [`RequestKind`] trait and the [`CompanyFacts`], [`NotesFile`], [`DailyIndex`],
//!   [`DailyIndexListing`], [`Submissions`], [`FilingIndex`], [`FilingDocument`], and
//!   [`FilingDocumentSet`] kinds.

use std::marker::PhantomData;

use reqwest::{Method, Request, Url};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::shared::filing_documents::{FilingDocumentRef, FilingDocumentSetRef};
use crate::shared::filing_ref::FilingRef;
use crate::shared::index_date::IndexDate;
use crate::shared::{cik::Cik, notes_period::NotesPeriod, request::SecRequest as SecRequestTrait};

pub mod builder;
pub mod constants;
pub mod kind;

pub use builder::SecRequestBuilder;
pub use kind::{
    CompanyFacts, DailyIndex, DailyIndexListing, FilingDocument, FilingDocumentSet, FilingIndex,
    NotesFile, RequestKind, StoredAs, Submissions,
};

use constants::{
    SEC_COMPANY_FACTS_URL_PREFIX, SEC_COMPANY_FACTS_URL_SUFFIX, SEC_DAILY_INDEX_LISTING_FILE_NAME,
    SEC_DAILY_INDEX_URL_PREFIX, SEC_FILING_FOLDER_URL_PREFIX, SEC_FILING_INDEX_FILE_NAME,
    SEC_NOTES_URL_PREFIX, SEC_SUBMISSIONS_URL_PREFIX, SEC_SUBMISSIONS_URL_SUFFIX,
};

/// A ready-to-send SEC API request of kind `K`.
///
/// Wraps a `reqwest::Request` built from a high-level [`SecRequestType`].
/// [`SecRequest::new`](crate::shared::request::SecRequest::new) sets the endpoint URL and HTTP
/// method for the kind. The kind stays on the type after the build:
///
/// - [`CompanyFacts`], the default: built from a [`Cik`].
/// - [`NotesFile`]: built from a [`NotesPeriod`].
/// - [`DailyIndex`] and [`DailyIndexListing`]: built from an [`IndexDate`].
/// - [`Submissions`]: built from a [`Cik`].
/// - [`FilingIndex`]: built from a [`FilingRef`].
/// - [`FilingDocument`]: built from a [`FilingDocumentRef`].
/// - [`FilingDocumentSet`]: built from a [`FilingDocumentSetRef`].
///
/// The request keeps the input it was built from. Equality, hashing, and ordering compare that
/// input. For a [`FilingIndex`] request, the form, the amendment flag, and the date filed count,
/// and the order follows the ten-digit [`Cik`].
#[derive(Debug)]
pub struct SecRequest<K: RequestKind = CompanyFacts> {
    /// The raw HTTP request type this struct wraps.
    pub inner: Request,
    input: K::Input,
    kind: PhantomData<K>,
}

impl<K: RequestKind> Serialize for SecRequest<K> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("SecRequest", 2)?;
        state.serialize_field("url", self.inner.url().as_str())?;
        state.serialize_field("method", self.inner.method().as_str())?;
        state.end()
    }
}

// `builder` is on `SecRequest<CompanyFacts>` only, so `SecRequest::builder()` compiles without
// naming a kind. The builder's kind methods set the kind.
impl SecRequest {
    /// Creates a new [`SecRequestBuilder`] for constructing an [`SecRequest`].
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
    pub const fn builder() -> SecRequestBuilder {
        SecRequestBuilder::new()
    }
}

impl<K: RequestKind> SecRequest<K> {
    /// Consumes the `SecRequest` and returns the inner `reqwest::Request`.
    #[must_use]
    pub fn into_inner(self) -> Request {
        self.inner
    }

    /// Returns the input the request was built from.
    #[must_use]
    pub const fn input(&self) -> &K::Input {
        &self.input
    }
}

impl<K: RequestKind> SecRequestTrait for SecRequest<K> {
    type Inner = Request;
    type RequestInput = K::Input;
    type Kind = K;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    /// Builds the request from the [`SecRequestType`] that `K` makes from `request_input`.
    ///
    /// The kind decides the input, so a request of one kind cannot be built from the input of
    /// another:
    ///
    /// ```compile_fail,E0308
    /// use sec::shared::cik::Cik;
    /// use sec::shared::request::SecRequest as SecRequestTrait;
    /// use sec::shared::request::implementations::sec_request::{NotesFile, SecRequest};
    ///
    /// let cik = Cik::new("1067983").expect("A hardcoded valid CIK should always parse");
    /// let _request = SecRequest::<NotesFile>::new(cik);
    /// ```
    fn new(request_input: Self::RequestInput) -> Self {
        let url = match K::request_type(request_input.clone()) {
            SecRequestType::FetchAllCompanyFacts { cik } => {
                format!("{SEC_COMPANY_FACTS_URL_PREFIX}{cik}{SEC_COMPANY_FACTS_URL_SUFFIX}")
            }
            SecRequestType::FetchNotesFile { period } => {
                format!("{SEC_NOTES_URL_PREFIX}{}", period.file_name())
            }
            SecRequestType::FetchDailyIndex { date } => format!(
                "{SEC_DAILY_INDEX_URL_PREFIX}{}/{}/form.{}.idx",
                date.year(),
                date.quarter(),
                date.basic_format()
            ),
            SecRequestType::FetchDailyIndexListing { date } => format!(
                "{SEC_DAILY_INDEX_URL_PREFIX}{}/{}/{SEC_DAILY_INDEX_LISTING_FILE_NAME}",
                date.year(),
                date.quarter()
            ),
            SecRequestType::FetchSubmissions { cik } => {
                format!("{SEC_SUBMISSIONS_URL_PREFIX}{cik}{SEC_SUBMISSIONS_URL_SUFFIX}")
            }
            SecRequestType::FetchFilingIndex { filing } => {
                format!("{}{SEC_FILING_INDEX_FILE_NAME}", filing_folder_url(&filing))
            }
            SecRequestType::FetchFilingDocument { document } => filing_document_url(&document),
            SecRequestType::FetchFilingDocumentSet { documents } => {
                filing_folder_url(documents.filing())
            }
        };
        let request = Request::new(
            Method::GET,
            Url::parse(&url)
                .expect("A hardcoded URL prefix joined with a validated input should always parse"),
        );
        Self {
            inner: request,
            input: request_input,
            kind: PhantomData,
        }
    }
}

/// Builds the URL of the folder of `filing`, with a slash at the end.
///
/// The URL takes only the ASCII digits of the accession number, so the dashes drop out.
fn filing_folder_url(filing: &FilingRef) -> String {
    let accession_digits: String = filing
        .accession_number()
        .value()
        .chars()
        .filter(char::is_ascii_digit)
        .collect();
    format!(
        "{SEC_FILING_FOLDER_URL_PREFIX}{}/{accession_digits}/",
        filing.cik().unpadded()
    )
}

/// Builds the URL of one file in the folder of a filing.
///
/// The file name becomes one path segment. A `/`, `?`, `#`, or `%` in it is percent-encoded, so
/// the URL stays in the folder of the filing. A file name of `..` adds no segment, so the URL
/// names the folder itself.
fn filing_document_url(document: &FilingDocumentRef) -> String {
    let mut url = Url::parse(&filing_folder_url(document.filing()))
        .expect("A hardcoded URL prefix joined with a validated filing should always parse");
    url.path_segments_mut()
        .expect("An HTTPS URL always has path segments")
        .pop_if_empty()
        .push(document.file_name());
    url.into()
}

impl<K: RequestKind> Clone for SecRequest<K> {
    fn clone(&self) -> Self {
        Self {
            inner: self
                .inner
                .try_clone()
                .expect("SEC requests have no streaming body, so cloning should always succeed"),
            input: self.input.clone(),
            kind: PhantomData,
        }
    }
}

impl<K: RequestKind> PartialEq for SecRequest<K> {
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input
    }
}

impl<K: RequestKind> Eq for SecRequest<K> {}

impl<K: RequestKind> std::hash::Hash for SecRequest<K> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.input.hash(state);
    }
}

impl<K: RequestKind> PartialOrd for SecRequest<K> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: RequestKind> Ord for SecRequest<K> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.input.cmp(&other.input)
    }
}

/// The high-level SEC request kinds that are supported.
///
/// Each variant carries the domain inputs that request needs. A [`RequestKind`] builds the variant,
/// and [`SecRequest::new`](crate::shared::request::SecRequest::new) derives the endpoint URL and
/// HTTP method from it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[non_exhaustive]
pub enum SecRequestType {
    /// Fetch a company's full Company Facts payload (all financial statement data) by CIK.
    FetchAllCompanyFacts {
        /// The CIK of the company to fetch.
        cik: Cik,
    },
    /// Fetch the SEC Financial Statement and Notes file of one period.
    FetchNotesFile {
        /// The period of the Notes file to fetch.
        period: NotesPeriod,
    },
    /// Fetch the daily index of one date.
    FetchDailyIndex {
        /// The date of the daily index to fetch.
        date: IndexDate,
    },
    /// Fetch the listing of the daily-index folder of the quarter of a date.
    FetchDailyIndexListing {
        /// A date of the quarter whose listing to fetch.
        date: IndexDate,
    },
    /// Fetch a company's `submissions` JSON (name, SIC code, tickers, and former names) by CIK.
    FetchSubmissions {
        /// The CIK of the company to fetch.
        cik: Cik,
    },
    /// Fetch the folder index of one filing. The URL holds the CIK without its leading zeros, as
    /// the archive serves the padded form only through a redirect.
    FetchFilingIndex {
        /// The filing whose folder index to fetch.
        filing: FilingRef,
    },
    /// Fetch one file of the folder of a filing. The URL holds the CIK without its leading zeros,
    /// as in [`FetchFilingIndex`](Self::FetchFilingIndex).
    FetchFilingDocument {
        /// The file to fetch.
        document: FilingDocumentRef,
    },
    /// Fetch the XBRL instance and the optional label file of a filing. The URL names the folder
    /// of the filing, as in [`FetchFilingIndex`](Self::FetchFilingIndex).
    FetchFilingDocumentSet {
        /// The files to fetch.
        documents: FilingDocumentSetRef,
    },
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use reqwest::{Method, Url};

    use chrono::NaiveDate;

    use super::{CompanyFacts, FilingIndex, NotesFile, RequestKind, SecRequest};
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::{cik::Cik, request::SecRequest as SecRequestTrait};

    fn filing_index_request(
        cik: &str,
        accession_number: &str,
        form: Form,
    ) -> SecRequest<FilingIndex> {
        let filing = FilingRef::new(
            Cik::new(cik).expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new(accession_number).expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(form, false),
            NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
        );
        SecRequest::builder()
            .filing_index()
            .filing_ref(filing)
            .build()
    }

    #[test]
    fn should_differ_when_two_filing_index_requests_share_the_url_but_not_the_form() {
        let ten_k = filing_index_request("3545", "0000003545-26-000047", Form::TenK);
        let ten_q = filing_index_request("3545", "0000003545-26-000047", Form::TenQ);

        let expected_result = false;

        let result = ten_k == ten_q;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_filing_index_requests_by_the_ten_digit_cik_when_the_urls_sort_the_other_way() {
        let apple = filing_index_request("320193", "0000320193-17-000070", Form::TenK);
        let berkshire = filing_index_request("1067983", "0001067983-26-000001", Form::TenK);

        let expected_result = true;

        let result = apple < berkshire;

        assert_eq!(result, expected_result);
    }

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

    fn build_for_kind<K: RequestKind>(input: K::Input) -> SecRequest<K> {
        SecRequest::<K>::new(input)
    }

    #[test]
    fn should_build_the_company_facts_url_when_generic_code_builds_a_company_facts_request() {
        let cik = Cik::new("0001234567")
            .expect("Given a hardcoded 10-digit CIK, Cik creation should always succeed");

        let expected_result =
            Url::parse("https://data.sec.gov/api/xbrl/companyfacts/CIK0001234567.json")
                .expect("Given a hardcoded HTTPS URL literal, Url parsing should always succeed");

        let result = build_for_kind::<CompanyFacts>(cik).inner().url().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_notes_url_when_generic_code_builds_a_notes_file_request() {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a published period, NotesPeriod creation should always succeed");

        let expected_result = Url::parse(
            "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2024q1_notes.zip",
        )
        .expect("Given a hardcoded HTTPS URL literal, Url parsing should always succeed");

        let result = build_for_kind::<NotesFile>(period).inner().url().clone();

        assert_eq!(result, expected_result);
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}
    const fn implements_unpin<T: Unpin>() {}

    #[test]
    const fn should_be_send_for_sec_request() {
        implements_send::<SecRequest>();
    }

    #[test]
    const fn should_be_sync_for_sec_request() {
        implements_sync::<SecRequest>();
    }

    #[test]
    const fn should_be_unpin_for_sec_request() {
        implements_unpin::<SecRequest>();
    }

    #[test]
    const fn should_be_send_for_notes_file_sec_request() {
        implements_send::<SecRequest<NotesFile>>();
    }

    #[test]
    const fn should_be_sync_for_notes_file_sec_request() {
        implements_sync::<SecRequest<NotesFile>>();
    }

    #[test]
    const fn should_be_unpin_for_notes_file_sec_request() {
        implements_unpin::<SecRequest<NotesFile>>();
    }
}
