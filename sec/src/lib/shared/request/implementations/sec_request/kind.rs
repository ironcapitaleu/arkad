//! # Request Kinds
//!
//! Provides the [`RequestKind`] trait and the request kinds a [`SecRequest`](super::SecRequest)
//! can carry.
//!
//! The kind names the input of a request, so a wrong input is a compile error. The kind stays on
//! the request after the build, so the client can tell a Company Facts request from a Notes
//! request by its type.

use std::fmt::Debug;

use crate::shared::cik::Cik;
use crate::shared::notes_period::NotesPeriod;

use super::SecRequestType;

/// The kind of a [`SecRequest`](super::SecRequest): the input it needs and the
/// [`SecRequestType`] it builds from that input.
///
/// A new kind implements this trait once and gets
/// [`SecRequest::new`](crate::shared::request::SecRequest::new) for free.
pub trait RequestKind: Debug + Send + Sync {
    /// The domain input of the request, for example a [`Cik`].
    type Input;

    /// Builds the [`SecRequestType`] of this kind from its input.
    fn request_type(input: Self::Input) -> SecRequestType;
}

/// The kind of a request that fetches the Company Facts of one company.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CompanyFacts;

impl RequestKind for CompanyFacts {
    type Input = Cik;

    fn request_type(cik: Cik) -> SecRequestType {
        SecRequestType::FetchAllCompanyFacts { cik }
    }
}

/// The kind of a request that downloads the Notes file of one period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NotesFile;

impl RequestKind for NotesFile {
    type Input = NotesPeriod;

    fn request_type(period: NotesPeriod) -> SecRequestType {
        SecRequestType::FetchNotesFile { period }
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
        let cik = Cik::new("0001234567").expect("Hardcoded CIK should be valid");

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
}
