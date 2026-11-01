//! # Transitions from `ExecuteSecRequest`
//!
//! Holds the transitions leaving the
//! [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest)
//! state.
//!
//! ## Modules
//!
//! - `request_check`: The check that the request matches the context, shared by the transitions
//!   out of `ExecuteSecRequest`.
//! - [`select_filing_documents`]: Transition of the folder index kind into the state that picks
//!   the XBRL instance and the label file.
//! - [`store_sec_response`]: Transition of every
//!   [`StoredAs`](crate::shared::request::implementations::sec_request::StoredAs) kind into the
//!   generic store state.
//! - [`store_filing_documents`]: Transition of the filing documents kind into the state that
//!   stores the XBRL instance and the label file.
//! - [`store_notes_archive`]: Transition of the Notes kind into the state that stores the Notes
//!   file.
//! - [`store_submissions`]: Transition of the Submissions kind into the state that stores the
//!   `submissions` JSON.

mod request_check;
pub mod select_filing_documents;
pub mod store_filing_documents;
pub mod store_notes_archive;
pub mod store_sec_response;
pub mod store_submissions;
