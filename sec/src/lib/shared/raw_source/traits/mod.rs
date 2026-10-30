//! # Raw Source Traits
//!
//! The contract that names the SEC file a
//! [`RawDocument`](crate::shared::raw_document::RawDocument) holds.
//!
//! ## Modules
//!
//! - [`raw_file_source`]: The [`RawFileSource`] trait for SEC files that arrive as raw bytes.
//! - [`raw_source`]: The [`RawSource`] trait.

pub mod raw_file_source;
pub mod raw_source;

pub use raw_file_source::RawFileSource;
pub use raw_source::RawSource;
