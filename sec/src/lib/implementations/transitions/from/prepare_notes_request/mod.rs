//! # Transitions from `PrepareNotesRequest`
//!
//! Holds the transitions leaving the
//! [`PrepareNotesRequest`](crate::implementations::states::extract::prepare_notes_request::PrepareNotesRequest)
//! state.
//!
//! ## Modules
//!
//! - [`download_notes_archive`]: Transition into the state that downloads the Notes file.

pub mod download_notes_archive;
