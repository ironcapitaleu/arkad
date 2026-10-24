//! # Raw Source Trait
//!
//! Provides [`RawSource`], the trait every SEC file that a
//! [`RawDocument`](crate::shared::raw_document::RawDocument) can hold implements.

use std::fmt::Debug;
use std::hash::Hash;

use crate::shared::document_kind::SecDocumentKind;

/// Names the SEC file that a [`RawDocument`](crate::shared::raw_document::RawDocument) holds.
///
/// Each SEC file is a marker type that implements this trait. Its [`RawSource::KIND`] becomes the
/// document's kind. Each marker must name its own [`SecDocumentKind`] variant, because the
/// repository compares kinds, not Rust types. Two markers with one kind read and write each other's
/// files.
pub trait RawSource: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash {
    /// The kind of document this file is.
    const KIND: SecDocumentKind;

    /// The version of the raw document's context for this file.
    const KIND_VERSION: u32;
}
