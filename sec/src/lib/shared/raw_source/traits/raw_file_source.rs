//! # Raw File Source Trait
//!
//! Provides [`RawFileSource`], the [`RawSource`] markers of SEC files that arrive as raw bytes.

use crate::shared::raw_source::traits::RawSource;

/// Marks a [`RawSource`] whose SEC response body is raw bytes, such as a ZIP or an `.idx` file.
///
/// Only these markers get
/// [`RawDocument::from_file`](crate::shared::raw_document::RawDocument::from_file). A JSON source
/// such as [`RawCompanyFacts`](crate::shared::raw_source::RawCompanyFacts) does not implement it,
/// so a raw-bytes body cannot be stored under its document type.
pub trait RawFileSource: RawSource {}
