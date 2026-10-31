//! # Filing Documents
//!
//! Provides [`FilingDocuments`], the XBRL instance and the optional label file of one filing,
//! [`FolderIndex`], the member names of the folder of the filing that they are picked from,
//! [`FilingDocumentRef`], which names one file of that folder, and [`FilingDocumentSetRef`], which
//! names the instance and the label file together.
//!
//! ## Modules
//!
//! - [`filing_document_ref`]: The [`FilingDocumentRef`], the input of a request for one file.
//! - [`filing_document_set_ref`]: The [`FilingDocumentSetRef`], the input of a request for the
//!   instance and the label file.
//! - [`filing_documents_error`]: The [`InvalidFolderIndex`] returned when an `index.json` does not
//!   parse, and the [`FilingDocumentsError`] returned when no single document matches a rule.
//! - [`folder_index`]: The [`FolderIndex`] and the rules of
//!   [`FolderIndex::select_documents`].

use serde::Serialize;

pub mod filing_document_ref;
pub mod filing_document_set_ref;
pub mod filing_documents_error;
pub mod folder_index;

pub use filing_document_ref::FilingDocumentRef;
pub use filing_document_set_ref::FilingDocumentSetRef;
pub use filing_documents_error::{FilingDocumentRole, FilingDocumentsError, InvalidFolderIndex};
pub use folder_index::FolderIndex;

/// The file names of the XBRL instance and the label file of one filing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct FilingDocuments {
    instance: String,
    label_file: Option<String>,
}

impl FilingDocuments {
    /// Creates a new [`FilingDocuments`] from the file names of the instance and the label file.
    #[must_use]
    pub fn new(instance: impl Into<String>, label_file: Option<String>) -> Self {
        Self {
            instance: instance.into(),
            label_file,
        }
    }

    /// Returns the file name of the XBRL instance.
    #[must_use]
    pub fn instance(&self) -> &str {
        &self.instance
    }

    /// Returns the file name of the label file, or `None` if the folder has none.
    #[must_use]
    pub fn label_file(&self) -> Option<&str> {
        self.label_file.as_deref()
    }
}
