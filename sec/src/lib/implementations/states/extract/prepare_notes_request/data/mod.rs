//! # Prepare Notes Request Data
//!
//! Groups the input and output data types for the
//! [`PrepareNotesRequest`](super::PrepareNotesRequest) state.
//!
//! ## Modules
//!
//! - [`input`]: The [`PrepareNotesRequestInput`] holding the validated [`NotesPeriod`](crate::shared::notes_period::NotesPeriod).
//! - [`output`]: The [`PrepareNotesRequestOutput`] holding the prepared [`SecRequest`](crate::shared::request::implementations::sec_request::SecRequest).

pub mod input;
pub mod output;

pub use input::PrepareNotesRequestInput;
pub use output::PrepareNotesRequestOutput;
