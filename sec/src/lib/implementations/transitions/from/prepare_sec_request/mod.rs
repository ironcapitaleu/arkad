//! # Transitions from `PrepareSecRequest`
//!
//! Holds the transitions leaving the
//! [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest)
//! state.
//!
//! ## Modules
//!
//! - [`execute_sec_request`]: Transition into the request-execution state.
//! - [`download_notes_archive`]: Transition of the Notes kind into the state that downloads the
//!   Notes file.

pub mod download_notes_archive;
pub mod execute_sec_request;
