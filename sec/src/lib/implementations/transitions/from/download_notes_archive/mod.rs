//! # Transitions from `DownloadNotesArchive`
//!
//! Holds the transitions leaving the
//! [`DownloadNotesArchive`](crate::implementations::states::extract::download_notes_archive::DownloadNotesArchive)
//! state.
//!
//! ## Modules
//!
//! - [`store_notes_archive`]: Transition into the state that stores the Notes file.

pub mod store_notes_archive;
