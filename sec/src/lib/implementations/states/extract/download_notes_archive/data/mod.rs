//! # Download Notes Archive Data
//!
//! Groups the input and output data types for the
//! [`DownloadNotesArchive`](super::DownloadNotesArchive) state.
//!
//! ## Modules
//!
//! - [`input`]: The [`DownloadNotesArchiveInput`] holding the prepared [`SecRequest`](crate::shared::request::implementations::sec_request::SecRequest).
//! - [`output`]: The [`DownloadNotesArchiveOutput`] holding the downloaded [`SecNotesFileResponse`](crate::shared::response::implementations::sec_notes_file_response::SecNotesFileResponse).

pub mod input;
pub mod output;

pub use input::DownloadNotesArchiveInput;
pub use output::DownloadNotesArchiveOutput;
