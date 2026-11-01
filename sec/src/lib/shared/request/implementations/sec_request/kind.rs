//! # Request Kinds
//!
//! Provides the [`RequestKind`] trait and the request kinds an [`SecRequest`](super::SecRequest)
//! can carry.
//!
//! The kind names the input of a request, so a wrong input is a compile error. The kind stays on
//! the request after the build, so the client can tell a Company Facts request from a Notes
//! request by its type.

use std::fmt::{Debug, Display};
use std::hash::Hash;

use serde::Serialize;

use crate::shared::cik::Cik;
use crate::shared::filing_documents::{FilingDocumentRef, FilingDocumentSetRef};
use crate::shared::filing_ref::FilingRef;
use crate::shared::http_client::HasResponse;
use crate::shared::index_date::IndexDate;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::RawSource;
use crate::shared::run_fields::RunFields;
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;
use crate::traits::state_machine::state::StateData;

use super::SecRequestType;

/// The kind of an [`SecRequest`](super::SecRequest): the input it needs and the
/// [`SecRequestType`] it builds from that input.
///
/// A new kind implements this trait, adds an [`SecRequestType`] variant, and adds the URL of that
/// variant in [`SecRequest::new`](crate::shared::request::SecRequest::new).
pub trait RequestKind: Debug + Clone + Eq + Ord + Hash + Send + Sync + Unpin {
    /// The domain input of the request, for example a [`Cik`].
    type Input: Debug + Display + Clone + Eq + Ord + Hash + Serialize + Send + Sync + Unpin;

    /// The name of the input in serialized data, for example `cik`.
    const INPUT_NAME: &'static str;

    /// Builds the [`SecRequestType`] of this kind from its input.
    fn request_type(input: Self::Input) -> SecRequestType;
}

/// The kind of a request that fetches the Company Facts of one company.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CompanyFacts;

impl RequestKind for CompanyFacts {
    type Input = Cik;

    const INPUT_NAME: &'static str = "cik";

    fn request_type(cik: Cik) -> SecRequestType {
        SecRequestType::FetchAllCompanyFacts { cik }
    }
}

/// The kind of a request that downloads the Notes file of one period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NotesFile;

impl RequestKind for NotesFile {
    type Input = NotesPeriod;

    const INPUT_NAME: &'static str = "notes_period";

    fn request_type(period: NotesPeriod) -> SecRequestType {
        SecRequestType::FetchNotesFile { period }
    }
}

/// The kind of a request that downloads the daily index of one date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DailyIndex;

impl RequestKind for DailyIndex {
    type Input = IndexDate;

    const INPUT_NAME: &'static str = "index_date";

    fn request_type(date: IndexDate) -> SecRequestType {
        SecRequestType::FetchDailyIndex { date }
    }
}

/// The kind of a request that fetches the listing of the daily-index folder of one quarter.
///
/// The input is any date of the quarter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DailyIndexListing;

impl RequestKind for DailyIndexListing {
    type Input = IndexDate;

    const INPUT_NAME: &'static str = "index_date";

    fn request_type(date: IndexDate) -> SecRequestType {
        SecRequestType::FetchDailyIndexListing { date }
    }
}

/// The kind of a request that fetches the `submissions` JSON of one company.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Submissions;

impl RequestKind for Submissions {
    type Input = Cik;

    const INPUT_NAME: &'static str = "cik";

    fn request_type(cik: Cik) -> SecRequestType {
        SecRequestType::FetchSubmissions { cik }
    }
}

/// The kind of a request that fetches the folder index of one filing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FilingIndex;

impl RequestKind for FilingIndex {
    type Input = FilingRef;

    const INPUT_NAME: &'static str = "filing_ref";

    fn request_type(filing: FilingRef) -> SecRequestType {
        SecRequestType::FetchFilingIndex { filing }
    }
}

/// The kind of a request that downloads one file of the folder of one filing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FilingDocument;

impl RequestKind for FilingDocument {
    type Input = FilingDocumentRef;

    const INPUT_NAME: &'static str = "filing_document";

    fn request_type(document: FilingDocumentRef) -> SecRequestType {
        SecRequestType::FetchFilingDocument { document }
    }
}

/// The kind of a request that downloads the XBRL instance and the optional label file of one
/// filing.
///
/// The request URL names the folder of the filing. The client fetches each file as one
/// [`FilingDocument`] request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FilingDocumentSet;

impl RequestKind for FilingDocumentSet {
    type Input = FilingDocumentSetRef;

    const INPUT_NAME: &'static str = "filing_document_set";

    fn request_type(documents: FilingDocumentSetRef) -> SecRequestType {
        SecRequestType::FetchFilingDocumentSet { documents }
    }
}

/// A request kind whose response the
/// [`StoreSecResponse`](crate::implementations::states::extract::store_sec_response::StoreSecResponse)
/// state keeps as a raw document of one [`RawSource`].
///
/// The kind names what differs between two store steps: the source, the state name, and the
/// document and output that [`Self::store`] builds. The state bounds its repository by
/// [`Self::Source`], so a repository of another source does not compile.
pub trait StoredAs: HasResponse {
    /// The source of the raw document that stores the response to a request of this kind.
    type Source: RawSource;

    /// The output of the store state, for example the key of the stored document.
    type Output: StateData + Display + Serialize;

    /// The name of the store state. The stored document records it as its producer.
    const STATE_NAME: &'static str;

    /// The name of the request input in error messages, for example `CIK`.
    const INPUT_LABEL: &'static str;

    /// Builds the raw document to write and the output the state reports once the write succeeds.
    fn store(
        response: &Self::Response,
        request_input: &Self::Input,
        requested_url: &Url,
        user_agent: &UserAgent,
        run: RunFields,
    ) -> (RawDocument<Self::Source>, Self::Output);
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::shared::cik::Cik;
    use crate::shared::filing_documents::{
        FilingDocumentRef, FilingDocumentSetRef, FilingDocuments,
    };
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::notes_period::NotesPeriod;

    use super::{
        CompanyFacts, DailyIndex, DailyIndexListing, FilingDocument, FilingDocumentSet,
        FilingIndex, NotesFile, RequestKind, SecRequestType, Submissions,
    };

    fn filing_ref() -> FilingRef {
        FilingRef::new(
            Cik::new("320193").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000320193-17-000070").expect("Given a hardcoded accession number of the right shape, creation should always succeed"),
            ParsedForm::new(Form::TenK, false),
            NaiveDate::from_ymd_opt(2017, 11, 3)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    #[test]
    fn should_return_fetch_all_company_facts_for_company_facts_kind() {
        let cik = Cik::new("0001234567")
            .expect("Given a hardcoded 10-digit CIK, Cik creation should always succeed");

        let expected_result = SecRequestType::FetchAllCompanyFacts { cik: cik.clone() };

        let result = CompanyFacts::request_type(cik);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_fetch_notes_file_for_notes_file_kind() {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a published period, NotesPeriod creation should always succeed");

        let expected_result = SecRequestType::FetchNotesFile { period };

        let result = NotesFile::request_type(period);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_fetch_submissions_for_submissions_kind() {
        let cik = Cik::new("0001234567")
            .expect("Given a hardcoded 10-digit CIK, Cik creation should always succeed");

        let expected_result = SecRequestType::FetchSubmissions { cik: cik.clone() };

        let result = Submissions::request_type(cik);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_fetch_filing_index_for_filing_index_kind() {
        let filing = filing_ref();

        let expected_result = SecRequestType::FetchFilingIndex {
            filing: filing_ref(),
        };

        let result = FilingIndex::request_type(filing);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_fetch_filing_document_for_filing_document_kind() {
        let document = FilingDocumentRef::new(filing_ref(), "aapl-20170930.xml");

        let expected_result = SecRequestType::FetchFilingDocument {
            document: FilingDocumentRef::new(filing_ref(), "aapl-20170930.xml"),
        };

        let result = FilingDocument::request_type(document);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_fetch_filing_document_set_for_filing_document_set_kind() {
        let documents = FilingDocumentSetRef::new(
            filing_ref(),
            FilingDocuments::new("aapl-20170930.xml", None),
        );

        let expected_result = SecRequestType::FetchFilingDocumentSet {
            documents: FilingDocumentSetRef::new(
                filing_ref(),
                FilingDocuments::new("aapl-20170930.xml", None),
            ),
        };

        let result = FilingDocumentSet::request_type(documents);

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_company_facts() {
        implements_auto_traits::<CompanyFacts>();
    }

    #[test]
    const fn should_implement_auto_traits_for_notes_file() {
        implements_auto_traits::<NotesFile>();
    }

    #[test]
    const fn should_implement_auto_traits_for_daily_index() {
        implements_auto_traits::<DailyIndex>();
    }

    #[test]
    const fn should_implement_auto_traits_for_daily_index_listing() {
        implements_auto_traits::<DailyIndexListing>();
    }

    #[test]
    const fn should_implement_auto_traits_for_submissions() {
        implements_auto_traits::<Submissions>();
    }

    #[test]
    const fn should_implement_auto_traits_for_filing_index() {
        implements_auto_traits::<FilingIndex>();
    }

    #[test]
    const fn should_implement_auto_traits_for_filing_document() {
        implements_auto_traits::<FilingDocument>();
    }

    #[test]
    const fn should_implement_auto_traits_for_filing_document_set() {
        implements_auto_traits::<FilingDocumentSet>();
    }
}
