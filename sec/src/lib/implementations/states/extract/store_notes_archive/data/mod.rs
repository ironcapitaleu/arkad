//! # Store Notes Archive Data
//!
//! Groups the input and output data types for the
//! [`StoreNotesArchive`](super::StoreNotesArchive) state.
//!
//! ## Modules
//!
//! - [`input`]: The [`StoreNotesArchiveInput`] holding the downloaded [`SecNotesFileResponse`](crate::shared::response::implementations::sec_notes_file_response::SecNotesFileResponse) and its [`NotesPeriod`](crate::shared::notes_period::NotesPeriod).
//! - [`output`]: The [`StoreNotesArchiveOutput`] holding the [`NotesArchiveRef`](crate::shared::notes_archive_ref::NotesArchiveRef) of the stored file.

pub mod input;
pub mod output;

pub use input::StoreNotesArchiveInput;
pub use output::StoreNotesArchiveOutput;
