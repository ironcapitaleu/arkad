//! # Document Type
//!
//! Provides [`SecDocumentType`], the types of document that the SEC state machines store.

use std::fmt::{self, Display, Formatter};

/// The types of document that the SEC state machines store.
///
/// Each variant's [`Display`] text is the `document_type` the store writes into a document's
/// metadata. The text must stay unique across every crate that stores documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecDocumentType {
    /// The Company Facts JSON exactly as the SEC returned it.
    RawCompanyFacts,
    /// The EDGAR daily index of one day exactly as the SEC returned it.
    RawDailyIndex,
    /// One Financial Statement and Notes ZIP exactly as the SEC returned it.
    RawNotesArchive,
    /// The `submissions` JSON of one company exactly as the SEC returned it.
    RawSubmissions,
    /// The XBRL instance of one filing exactly as the SEC returned it.
    RawXbrlInstance,
    /// The `_lab.xml` label file of one filing exactly as the SEC returned it.
    RawXbrlLabels,
}

impl Display for SecDocumentType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::RawCompanyFacts => write!(f, "raw_company_facts"),
            Self::RawDailyIndex => write!(f, "raw_daily_index"),
            Self::RawNotesArchive => write!(f, "raw_notes_archive"),
            Self::RawSubmissions => write!(f, "raw_submissions"),
            Self::RawXbrlInstance => write!(f, "raw_xbrl_instance"),
            Self::RawXbrlLabels => write!(f, "raw_xbrl_labels"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_sec_document_type() {
        implements_auto_traits::<SecDocumentType>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_sec_document_type() {
        implements_value_traits::<SecDocumentType>();
    }

    #[test]
    fn should_display_raw_company_facts_for_the_raw_company_facts_document_type() {
        let document_type = SecDocumentType::RawCompanyFacts;

        let expected_result = "raw_company_facts";

        let result = document_type.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_raw_daily_index_for_the_raw_daily_index_document_type() {
        let document_type = SecDocumentType::RawDailyIndex;

        let expected_result = "raw_daily_index";

        let result = document_type.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_raw_notes_archive_for_the_raw_notes_archive_document_type() {
        let document_type = SecDocumentType::RawNotesArchive;

        let expected_result = "raw_notes_archive";

        let result = document_type.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_raw_submissions_for_the_raw_submissions_document_type() {
        let document_type = SecDocumentType::RawSubmissions;

        let expected_result = "raw_submissions";

        let result = document_type.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_raw_xbrl_instance_for_the_raw_xbrl_instance_document_type() {
        let document_type = SecDocumentType::RawXbrlInstance;

        let expected_result = "raw_xbrl_instance";

        let result = document_type.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_raw_xbrl_labels_for_the_raw_xbrl_labels_document_type() {
        let document_type = SecDocumentType::RawXbrlLabels;

        let expected_result = "raw_xbrl_labels";

        let result = document_type.to_string();

        assert_eq!(result, expected_result);
    }
}
