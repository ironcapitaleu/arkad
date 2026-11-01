//! # Transitions from `StoreFilingIndex`
//!
//! Holds the transitions leaving the
//! [`StoreFilingIndex`](crate::implementations::states::extract::store_filing_index::StoreFilingIndex)
//! state.
//!
//! ## Modules
//!
//! - [`select_filing_documents`]: Transition into the state that picks the XBRL instance and the
//!   label file from the stored folder index.

pub mod select_filing_documents;
