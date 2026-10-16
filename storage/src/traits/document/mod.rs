//! # Document
//!
//! Provides [`Document`], the trait every storable document kind implements.

use crate::shared::document_key::DocumentKey;
use crate::traits::document_metadata::DocumentMetadata;

/// Defines what a store needs from a document to write it and to read it back.
///
/// A document is a key, the bytes stored under it, and the metadata stored beside them. Each
/// document kind is its own type and implements this trait, so one store holds any kind. The bytes
/// are the content exactly as stored. A kind with typed content serializes it into the bytes.
///
/// # Associated Types
///
/// - `Metadata`: The metadata kind stored beside the document.
pub trait Document: Send + Sized {
    /// The metadata kind stored beside the document.
    type Metadata: DocumentMetadata;

    /// Returns the key that names this document in the store.
    fn key(&self) -> &DocumentKey;

    /// Returns the document content, byte for byte as the store writes it.
    fn bytes(&self) -> &[u8];

    /// Returns the metadata stored beside the document.
    fn metadata(&self) -> &Self::Metadata;

    /// Builds the document from its key, its bytes, and its metadata.
    ///
    /// The implementor keeps the parts as given and checks nothing, so building cannot fail.
    fn from_parts(key: DocumentKey, bytes: Vec<u8>, metadata: Self::Metadata) -> Self;
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
