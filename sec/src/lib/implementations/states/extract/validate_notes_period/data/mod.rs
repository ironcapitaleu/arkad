//! # Validate Notes Period Data
//!
//! Groups the input and output data types for the
//! [`ValidateNotesPeriod`](super::ValidateNotesPeriod) state.
//!
//! ## Modules
//!
//! - [`input`]: The [`ValidateNotesPeriodInput`] holding the raw period string.
//! - [`output`]: The [`ValidateNotesPeriodOutput`] holding the validated [`NotesPeriod`](crate::shared::notes_period::NotesPeriod).

pub mod input;
pub mod output;

pub use input::ValidateNotesPeriodInput;
pub use output::ValidateNotesPeriodOutput;
