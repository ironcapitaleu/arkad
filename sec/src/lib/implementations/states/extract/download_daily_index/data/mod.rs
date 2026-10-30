//! # Download Daily Index Data
//!
//! Groups the input and output data types for the
//! [`DownloadDailyIndex`](super::DownloadDailyIndex) state.
//!
//! ## Modules
//!
//! - [`input`]: The [`DownloadDailyIndexInput`] holding the prepared [`SecRequest`](crate::shared::request::implementations::sec_request::SecRequest).
//! - [`output`]: The [`DownloadDailyIndexOutput`] holding the downloaded [`SecResponse<ZipBody>`](crate::shared::response::implementations::sec_response::SecResponse), if the day had filings.

pub mod input;
pub mod output;

pub use input::DownloadDailyIndexInput;
pub use output::DownloadDailyIndexOutput;
