//! # Transitions from `ExecuteSecRequest`
//!
//! Holds the transitions leaving the
//! [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest)
//! state.
//!
//! ## Modules
//!
//! - `request_check`: The check that the request matches the context, shared by the store
//!   transitions.
//! - [`store_company_facts`]: Transition into the state that stores the Company Facts.
//! - [`store_notes_archive`]: Transition of the Notes kind into the state that stores the Notes
//!   file.
//! - [`store_submissions`]: Transition of the Submissions kind into the state that stores the
//!   `submissions` JSON.

mod request_check;
pub mod store_company_facts;
pub mod store_notes_archive;
pub mod store_submissions;
