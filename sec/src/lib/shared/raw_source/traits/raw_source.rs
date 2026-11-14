//! # Raw Source Trait
//!
//! Provides [`RawSource`], the trait every SEC file that a
//! [`RawDocument`](crate::shared::raw_document::RawDocument) can hold implements.

use std::fmt::Debug;
use std::hash::Hash;

/// Names the SEC file that a [`RawDocument`](crate::shared::raw_document::RawDocument) holds.
///
/// Each SEC file is a marker type that implements this trait. Its [`RawSource::DOCUMENT_TYPE`]
/// becomes the document type of the stored document. Each marker must hold its own text, because
/// the repository compares document types, not Rust types. Two markers with one document type read
/// and write each other's files.
pub trait RawSource: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash {
    /// The document type of this file, as the store writes it into the metadata.
    ///
    /// The text must stay unique across every crate that stores documents. A change to the text
    /// makes each stored file of this source fail to read.
    const DOCUMENT_TYPE: &'static str;

    /// The version of the raw document's context for this file.
    const DOCUMENT_TYPE_VERSION: u32;
}
