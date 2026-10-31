//! # Store Filing Documents Data
//!
//! Groups the input and output data types for the
//! [`StoreFilingDocuments`](super::StoreFilingDocuments) state.
//!
//! ## Modules
//!
//! - [`input`]: The [`StoreFilingDocumentsInput`] holding the downloaded [`FilingDocumentResponses`](crate::shared::filing_documents::FilingDocumentResponses) and their [`FilingRef`](crate::shared::filing_ref::FilingRef).
//! - [`output`]: The [`StoreFilingDocumentsOutput`] holding the [`FilingDocumentsRef`](crate::shared::filing_documents_ref::FilingDocumentsRef) of the stored files.

pub mod input;
pub mod output;

pub use input::StoreFilingDocumentsInput;
pub use output::StoreFilingDocumentsOutput;
