//! # Document
//!
//! Provides [`Document`], the trait every document kind implements to be stored as a file.

use std::fmt;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::implementations::repository::filesystem::shared::document_key::DocumentKey;
use crate::implementations::repository::filesystem::shared::document_metadata::DocumentMetadata;

/// Defines what the store needs from a document to write it as a file and to read it back.
///
/// A document is a key, the bytes stored under it, and the metadata stored beside them. Each
/// document kind is its own type and implements this trait.
///
/// The crate that owns a kind defines an enum of its kinds and sets [`Document::Kind`] to it.
/// [`Document::KIND`] names the variant of this type, and [`Document::KIND_VERSION`] the version
/// of its [`Document::Context`]. [`DocumentMetadataBuilder::build`] stores both, so a reader can
/// tell a file of another kind or version apart.
///
/// # Associated Types
///
/// - `Kind`: The enum of document kinds in the crate that owns this kind.
/// - `Context`: The metadata fields that only this document kind has.
///
/// # Examples
///
/// ```rust
/// use std::fmt;
///
/// use serde::{Deserialize, Serialize};
/// use storage::implementations::repository::filesystem::{Document, DocumentKey, DocumentMetadata};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum ReportKind {
///     Summary,
/// }
///
/// impl fmt::Display for ReportKind {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         match self {
///             Self::Summary => write!(f, "summary"),
///         }
///     }
/// }
///
/// #[derive(Serialize, Deserialize)]
/// struct SummaryContext {
///     source: String,
/// }
///
/// struct Summary {
///     key: DocumentKey,
///     bytes: Vec<u8>,
///     metadata: DocumentMetadata<SummaryContext>,
/// }
///
/// impl Document for Summary {
///     type Kind = ReportKind;
///     const KIND: ReportKind = ReportKind::Summary;
///     const KIND_VERSION: u32 = 1;
///     type Context = SummaryContext;
///
///     fn key(&self) -> &DocumentKey {
///         &self.key
///     }
///
///     fn bytes(&self) -> &[u8] {
///         &self.bytes
///     }
///
///     fn metadata(&self) -> &DocumentMetadata<SummaryContext> {
///         &self.metadata
///     }
///
///     fn from_parts(
///         key: DocumentKey,
///         bytes: Vec<u8>,
///         metadata: DocumentMetadata<SummaryContext>,
///     ) -> Self {
///         Self { key, bytes, metadata }
///     }
/// }
///
/// let expected_result = "summary";
///
/// let result = Summary::KIND.to_string();
///
/// assert_eq!(result, expected_result);
/// ```
///
/// [`DocumentMetadataBuilder::build`]: crate::implementations::repository::filesystem::DocumentMetadataBuilder::build
pub trait Document: Send + Sized {
    /// The enum of document kinds in the crate that owns this kind.
    ///
    /// The store writes its `Display` text as `kind`. The text must be unique across every crate
    /// that stores documents, because the store cannot tell two equal texts apart.
    type Kind: fmt::Display;

    /// The variant of [`Document::Kind`] that names this document kind.
    const KIND: Self::Kind;

    /// The version of [`Document::Context`].
    ///
    /// Raise it when a context field is removed, renamed, or redefined.
    const KIND_VERSION: u32;

    /// The metadata fields that only this document kind has.
    type Context: Serialize + DeserializeOwned;

    /// Returns the key that names this document in the store.
    fn key(&self) -> &DocumentKey;

    /// Returns the document content, byte for byte as the store writes it.
    fn bytes(&self) -> &[u8];

    /// Returns the metadata stored beside the document.
    fn metadata(&self) -> &DocumentMetadata<Self::Context>;

    /// Builds the document from its key, its bytes, and its metadata.
    ///
    /// The implementor keeps the parts as given and checks nothing, so building cannot fail.
    fn from_parts(
        key: DocumentKey,
        bytes: Vec<u8>,
        metadata: DocumentMetadata<Self::Context>,
    ) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::sample_document::SampleDocument;

    const fn implements_document<T: Document>() {}
    #[test]
    const fn should_implement_document_for_sample_document() {
        implements_document::<SampleDocument>();
    }
}
