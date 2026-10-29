//! # Document
//!
//! Provides [`Document`], the trait every document type implements to be stored as a file.

use std::fmt;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::implementations::repository::filesystem::shared::document_key::DocumentKey;
use crate::implementations::repository::filesystem::shared::document_metadata::DocumentMetadata;

/// Defines what the store needs from a document to write it as a file and to read it back.
///
/// A document is a key, the bytes stored under it, and the metadata stored beside them. Each
/// document type is its own Rust type and implements this trait.
///
/// The crate that owns a document type defines an enum of its document types and sets
/// [`Document::DocumentType`] to it. [`Document::DOCUMENT_TYPE`] names the variant of this type,
/// and [`Document::DOCUMENT_TYPE_VERSION`] the version of its [`Document::Context`].
/// [`DocumentMetadataBuilder::build`] stores both, so a reader can tell a file of another document
/// type or version apart.
///
/// # Associated Types
///
/// - `DocumentType`: The enum of document types in the crate that owns this document type.
/// - `IngestionMode`: The type that names the ingestion mode of the run that produced a document.
/// - `Context`: The metadata fields that only this document type has.
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
/// enum ReportType {
///     Summary,
/// }
///
/// impl fmt::Display for ReportType {
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
///     type DocumentType = ReportType;
///     type IngestionMode = &'static str;
///     const DOCUMENT_TYPE: ReportType = ReportType::Summary;
///     const DOCUMENT_TYPE_VERSION: u32 = 1;
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
/// let result = Summary::DOCUMENT_TYPE.to_string();
///
/// assert_eq!(result, expected_result);
/// ```
///
/// [`DocumentMetadataBuilder::build`]: crate::implementations::repository::filesystem::DocumentMetadataBuilder::build
pub trait Document: Send + Sized {
    /// The enum of document types in the crate that owns this document type.
    ///
    /// The store writes its `Display` text as `document_type`. The text must be unique across every
    /// crate that stores documents, because the store cannot tell two equal texts apart.
    type DocumentType: fmt::Display;

    /// The type that names the ingestion mode of the run that produced a document, for example an
    /// enum with one variant per mode.
    ///
    /// [`DocumentMetadataBuilder::ingestion_mode`] takes a value of this type and stores its
    /// `Display` text as `ingestion_mode`. So a document type accepts only its own modes. A mode is
    /// a small value, so the type is `Copy` and the builder takes it by value.
    ///
    /// [`DocumentMetadataBuilder::ingestion_mode`]: crate::implementations::repository::filesystem::DocumentMetadataBuilder::ingestion_mode
    type IngestionMode: fmt::Display + Copy;

    /// The variant of [`Document::DocumentType`] that names this document type.
    const DOCUMENT_TYPE: Self::DocumentType;

    /// The version of [`Document::Context`].
    ///
    /// Raise it when a context field is removed, renamed, or redefined.
    const DOCUMENT_TYPE_VERSION: u32;

    /// The metadata fields that only this document type has.
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
