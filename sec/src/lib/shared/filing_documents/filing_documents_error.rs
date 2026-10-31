//! # Filing Documents Errors
//!
//! Provides the [`InvalidFolderIndex`] returned when an `index.json` does not parse into a
//! [`FolderIndex`](super::FolderIndex), the [`FilingDocumentsError`] returned by
//! [`FolderIndex::select_documents`](super::FolderIndex::select_documents), and the
//! [`FilingDocumentRole`] it names when a folder holds more than one candidate.

use std::fmt::{self, Display, Formatter};

use thiserror::Error;

use crate::shared::financial::accession_number::AccessionNumber;

/// Error indicating that the `index.json` of a filing folder does not parse.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidFolderIndex] The folder index at '{url}' does not parse, Reason: '{reason}'")]
pub struct InvalidFolderIndex {
    /// The URL of the folder index, which names the CIK and the accession number.
    pub url: String,
    /// The message of the JSON parser.
    pub reason: String,
}

/// Error indicating that the folder index of a filing names no single XBRL instance or label file.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FilingDocumentsError {
    /// No member of the folder matches a rule for the XBRL instance.
    #[error(
        "[MissingXbrlInstance] The folder of filing '{accession_number}' holds no XBRL instance"
    )]
    MissingXbrlInstance {
        /// The accession number of the filing.
        accession_number: AccessionNumber,
    },
    /// More than one member of the folder matches the rule for one document.
    #[error(
        "[AmbiguousFilingDocument] The folder of filing '{accession_number}' holds {candidate_count} candidates for the {role}"
    )]
    AmbiguousFilingDocument {
        /// The accession number of the filing.
        accession_number: AccessionNumber,
        /// The document with more than one candidate.
        role: FilingDocumentRole,
        /// The number of members that match the rule.
        candidate_count: usize,
    },
}

/// The role of a document that [`FolderIndex::select_documents`](super::FolderIndex::select_documents)
/// picks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FilingDocumentRole {
    /// The XBRL instance, which holds the facts.
    XbrlInstance,
    /// The `_lab.xml` label file.
    LabelFile,
}

impl Display for FilingDocumentRole {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::XbrlInstance => write!(f, "XBRL instance"),
            Self::LabelFile => write!(f, "label file"),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_name_the_accession_number_when_missing_xbrl_instance_is_displayed() {
        let error = FilingDocumentsError::MissingXbrlInstance {
            accession_number: AccessionNumber::new("0000320193-17-000070"),
        };

        let expected_result = "[MissingXbrlInstance] The folder of filing '0000320193-17-000070' holds no XBRL instance";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_role_and_the_count_when_ambiguous_filing_document_is_displayed() {
        let error = FilingDocumentsError::AmbiguousFilingDocument {
            accession_number: AccessionNumber::new("0000320193-17-000070"),
            role: FilingDocumentRole::LabelFile,
            candidate_count: 2,
        };

        let expected_result = "[AmbiguousFilingDocument] The folder of filing '0000320193-17-000070' holds 2 candidates for the label file";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_url_and_quote_the_reason_when_invalid_folder_index_is_displayed() {
        let error = InvalidFolderIndex {
            url: "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json"
                .to_string(),
            reason: "EOF while parsing a value at line 1 column 0".to_string(),
        };

        let expected_result = "[InvalidFolderIndex] The folder index at 'https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json' does not parse, Reason: 'EOF while parsing a value at line 1 column 0'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
