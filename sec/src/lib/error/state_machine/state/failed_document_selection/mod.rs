//! # Failed Document Selection Error
//!
//! Provides the [`FailedDocumentSelection`] error: a domain-level [`FilingDocumentsError`] with
//! the name of the state in which the selection of the filing documents failed.
//!
//! ## Example
//!
//! ```rust
//! use sec::error::state_machine::state::failed_document_selection::FailedDocumentSelection;
//! use sec::shared::filing_documents::FilingDocumentsError;
//! use sec::shared::financial::accession_number::AccessionNumber;
//!
//! let filing_documents_error = FilingDocumentsError::MissingXbrlInstance {
//!     accession_number: AccessionNumber::new("0000003545-26-000047"),
//! };
//! let state_error = FailedDocumentSelection::new("Select Filing Documents", filing_documents_error);
//!
//! let expected_result = "Select Filing Documents";
//!
//! let result = state_error.state_name.as_str();
//!
//! assert_eq!(result, expected_result);
//! ```

use thiserror::Error;

use super::State as StateError;
use crate::shared::filing_documents::FilingDocumentsError;
use crate::traits::error::FromDomainError;

/// Error for a folder index that does not parse, or names no single XBRL instance or label file,
/// tagged with the state it occurred in.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[FailedDocumentSelection] Failure in State: '{state_name}', Caused by: {filing_documents_error}"
)]
pub struct FailedDocumentSelection {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level selection error.
    #[source]
    pub filing_documents_error: FilingDocumentsError,
}

impl FailedDocumentSelection {
    /// Creates a new state-level [`FailedDocumentSelection`] error.
    #[must_use]
    pub fn new(state_name: impl Into<String>, domain_error: FilingDocumentsError) -> Self {
        Self {
            state_name: state_name.into(),
            filing_documents_error: domain_error,
        }
    }
}

impl From<FailedDocumentSelection> for StateError {
    /// Converts a [`FailedDocumentSelection`] into a [`StateError::FailedDocumentSelection`]
    /// variant.
    fn from(domain_error: FailedDocumentSelection) -> Self {
        Self::FailedDocumentSelection(domain_error)
    }
}

impl FromDomainError<FilingDocumentsError> for FailedDocumentSelection {
    type DomainErr = FilingDocumentsError;

    /// Converts a domain-level [`FilingDocumentsError`] into a state-level
    /// [`FailedDocumentSelection`] error.
    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::financial::accession_number::AccessionNumber;

    fn missing_xbrl_instance() -> FilingDocumentsError {
        FilingDocumentsError::MissingXbrlInstance {
            accession_number: AccessionNumber::new("0000003545-26-000047"),
        }
    }

    #[test]
    fn should_convert_to_state_error_when_into_is_called() {
        let failed_document_selection =
            FailedDocumentSelection::from_domain_error("TestState", missing_xbrl_instance());

        let expected_result =
            StateError::FailedDocumentSelection(failed_document_selection.clone());

        let result: StateError = failed_document_selection.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_filing_documents_error_as_source_of_failed_document_selection() {
        let failed_document_selection =
            FailedDocumentSelection::new("TestState", missing_xbrl_instance());

        let expected_result = Some(&missing_xbrl_instance());

        let result = std::error::Error::source(&failed_document_selection)
            .and_then(|source| source.downcast_ref::<FilingDocumentsError>());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_the_error_chain_when_displayed() {
        let failed_document_selection =
            FailedDocumentSelection::new("Select Filing Documents", missing_xbrl_instance());

        let expected_result = "[FailedDocumentSelection] Failure in State: 'Select Filing Documents', Caused by: [MissingXbrlInstance] The folder of filing '0000003545-26-000047' holds no XBRL instance";

        let result = failed_document_selection.to_string();

        assert_eq!(result, expected_result);
    }
}
