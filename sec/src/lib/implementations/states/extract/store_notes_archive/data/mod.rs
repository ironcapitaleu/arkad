//! # Store Notes Archive Data
//!
//! Groups the output data type of the [`StoreNotesArchive`](super::StoreNotesArchive) state.
//!
//! The input data type is the generic
//! [`StoreSecResponseInput`](crate::implementations::states::extract::store_sec_response::StoreSecResponseInput).
//!
//! ## Modules
//!
//! - [`output`]: The [`StoreNotesArchiveOutput`] holding the [`NotesArchiveRef`](crate::shared::notes_archive_ref::NotesArchiveRef) of the stored file.

pub mod output;

pub use output::StoreNotesArchiveOutput;
