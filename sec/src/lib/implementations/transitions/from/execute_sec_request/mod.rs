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
//! - [`store_sec_response`]: Transition of every
//!   [`StoredAs`](crate::shared::request::implementations::sec_request::StoredAs) kind into the
//!   generic store state.
//! - [`store_filing_documents`]: Transition of the filing documents kind into the state that
//!   stores the XBRL instance and the label file.

mod request_check;
pub mod store_filing_documents;
pub mod store_sec_response;
