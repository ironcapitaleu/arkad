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
use crate::shared::index_date::IndexDate;
use crate::shared::notes_period::NotesPeriod;

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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::shared::cik::Cik;
    use crate::shared::notes_period::NotesPeriod;

    use super::{CompanyFacts, NotesFile, RequestKind, SecRequestType};

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
        implements_auto_traits::<super::DailyIndex>();
    }

    #[test]
    const fn should_implement_auto_traits_for_daily_index_listing() {
        implements_auto_traits::<super::DailyIndexListing>();
    }
}
