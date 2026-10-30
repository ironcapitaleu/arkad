//! # Raw File Source Trait
//!
//! Provides [`RawFileSource`], the [`RawSource`] markers of SEC files that arrive as raw bytes.

use crate::shared::raw_source::traits::RawSource;

/// Marks a [`RawSource`] whose SEC response body is raw bytes, such as a ZIP or an `.idx` file.
///
/// Gates [`RawDocument::from_file`](crate::shared::raw_document::RawDocument::from_file). A JSON
/// source must not implement it.
pub trait RawFileSource: RawSource {}
