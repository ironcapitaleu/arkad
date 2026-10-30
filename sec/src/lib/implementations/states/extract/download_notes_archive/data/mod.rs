//! # Download Notes Archive Data
//!
//! Groups the input and output data types for the
//! [`DownloadNotesArchive`](super::DownloadNotesArchive) state.
//!
//! ## Modules
//!
//! - [`input`]: The [`DownloadNotesArchiveInput`] holding the prepared [`SecRequest`](crate::shared::request::implementations::sec_request::SecRequest).
//! - [`output`]: The [`DownloadNotesArchiveOutput`] holding the downloaded [`SecResponse<ZipBody>`](crate::shared::response::implementations::sec_response::SecResponse).

pub mod input;
pub mod output;

pub use input::DownloadNotesArchiveInput;
pub use output::DownloadNotesArchiveOutput;
