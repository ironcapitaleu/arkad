//! # Listed Document
//!
//! Provides [`ListedDocument`], one document that a listing of a
//! [`FilesystemRepository`](crate::implementations::repository::filesystem::FilesystemRepository)
//! yields.

use chrono::{DateTime, Utc};

use crate::implementations::repository::filesystem::shared::document_key::DocumentKey;
use crate::implementations::repository::filesystem::shared::document_metadata::DocumentMetadata;
use crate::implementations::repository::filesystem::shared::stored_document::StoredDocument;

/// One listed document: its key, its metadata, and what the store found for its document file.
///
/// A listing reads the metadata, never the bytes of the document. So it does not check the size
/// and the SHA-256 digest that the metadata names. A caller compares
/// [`DocumentMetadata::size_in_bytes`] with the size in [`StoredDocument::Present`], and calls
/// [`get`](crate::ReadRepository::get) to check the digest.
///
/// # Type Parameters
///
/// - `C`: The metadata fields that only this document type has, the
///   [`Document::Context`](crate::implementations::repository::filesystem::Document::Context).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedDocument<C> {
    key: DocumentKey,
    metadata: DocumentMetadata<C>,
    metadata_modified_at: DateTime<Utc>,
    document: StoredDocument,
}

impl<C> ListedDocument<C> {
    /// Creates a new [`ListedDocument`] from its parts.
    pub(crate) const fn new(
        key: DocumentKey,
        metadata: DocumentMetadata<C>,
        metadata_modified_at: DateTime<Utc>,
        document: StoredDocument,
    ) -> Self {
        Self {
            key,
            metadata,
            metadata_modified_at,
            document,
        }
    }

    /// Returns the key of the document.
    #[must_use]
    pub const fn key(&self) -> &DocumentKey {
        &self.key
    }

    /// Returns the metadata stored beside the document.
    #[must_use]
    pub const fn metadata(&self) -> &DocumentMetadata<C> {
        &self.metadata
    }

    /// Returns the time the metadata file was last modified.
    ///
    /// A write writes the metadata before the document. So for a large document, this time is
    /// older than the end of the write by the length of the document upload. A caller that finds
    /// [`StoredDocument::Missing`], or a size that differs from the metadata, waits a grace period
    /// from this time before it counts the entry as a fault.
    #[must_use]
    pub const fn metadata_modified_at(&self) -> DateTime<Utc> {
        self.metadata_modified_at
    }

    /// Returns what the store found in place of the document file.
    #[must_use]
    pub const fn document(&self) -> StoredDocument {
        self.document
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_listed_document() {
        implements_auto_traits::<ListedDocument<()>>();
    }
}
