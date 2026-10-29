//! # Transitions from `ValidateNotesPeriod`
//!
//! Holds the transitions leaving the
//! [`ValidateNotesPeriod`](crate::implementations::states::extract::validate_notes_period::ValidateNotesPeriod)
//! state.
//!
//! ## Modules
//!
//! - [`prepare_notes_request`]: Transition into the state that builds the Notes request.

pub mod prepare_notes_request;
