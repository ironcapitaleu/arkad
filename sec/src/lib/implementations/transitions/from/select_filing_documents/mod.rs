//! # Transitions from `SelectFilingDocuments`
//!
//! Holds the transitions leaving the
//! [`SelectFilingDocuments`](crate::implementations::states::extract::select_filing_documents::SelectFilingDocuments)
//! state.
//!
//! ## Modules
//!
//! - [`prepare_sec_request`]: Transition into the state that prepares the request for the
//!   selected filing documents.

pub mod prepare_sec_request;
