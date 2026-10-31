//! # Select Filing Documents Data
//!
//! Groups the input and output data types for the
//! [`SelectFilingDocuments`](super::SelectFilingDocuments) state.
//!
//! ## Modules
//!
//! - [`input`]: The [`SelectFilingDocumentsInput`] holding the folder index of one filing.
//! - [`output`]: The [`SelectFilingDocumentsOutput`] holding the selected
//!   [`FilingDocuments`](crate::shared::filing_documents::FilingDocuments).

pub mod input;
pub mod output;

pub use input::SelectFilingDocumentsInput;
pub use output::SelectFilingDocumentsOutput;
