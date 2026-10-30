//! # Filesystem Repository Integration Tests
//!
//! Persists a record through [`FilesystemRepository`] and reads the same record from a real
//! directory on disk.
//!
//! ## External Dependencies
//!
//! - A writable system temp directory. Each test creates its own root there and deletes it at the
//!   end.

use std::fmt;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use uuid::Uuid;

use storage::implementations::repository::filesystem::{
    Document, DocumentKey, DocumentMetadata, FilesystemRepository, IngestionMode, Producer,
};
use storage::{ReadRepository, WriteRepository};

/// The document types of this test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SampleDocumentType {
    SampleDocument,
}

impl fmt::Display for SampleDocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sample_document")
    }
}

/// A document type that exists only for this test, with an empty context.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SampleDocument {
    key: DocumentKey,
    bytes: Bytes,
    metadata: DocumentMetadata<()>,
}

impl Document for SampleDocument {
    type DocumentType = SampleDocumentType;
    const DOCUMENT_TYPE: SampleDocumentType = SampleDocumentType::SampleDocument;
    const DOCUMENT_TYPE_VERSION: u32 = 1;
    type Context = ();

    fn key(&self) -> &DocumentKey {
        &self.key
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn metadata(&self) -> &DocumentMetadata<()> {
        &self.metadata
    }

    fn from_parts(
        key: DocumentKey,
        bytes: impl Into<Bytes>,
        metadata: DocumentMetadata<()>,
    ) -> Self {
        Self {
            key,
            bytes: bytes.into(),
            metadata,
        }
    }
}

fn sample_document() -> SampleDocument {
    let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
        .expect("Given a valid relative path, the key should always build");
    let bytes = br#"{"cik":320193, "n":1.50, "dup":1,"dup":2}"#;
    let metadata = DocumentMetadata::builder()
        .document_type::<SampleDocument>()
        .media_type("application/json")
        .content(bytes)
        .produced_at(DateTime::<Utc>::UNIX_EPOCH)
        .run(
            Uuid::nil(),
            Producer::new("Sample State", "0.1.0", "63c1f59"),
        )
        .ingestion_mode(IngestionMode::Backfill)
        .context(())
        .build();

    SampleDocument::from_parts(key, bytes.to_vec(), metadata)
}

#[tokio::test]
async fn should_return_the_same_record_when_reading_back_a_persisted_record_by_key() {
    let root = TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    );
    let repository = FilesystemRepository::builder()
        .read_write()
        .document_type::<SampleDocument>()
        .root(root.path())
        .build();
    let record = sample_document();
    let key = record.key().clone();

    let expected_result = Some(record.clone());

    repository
        .persist(record)
        .await
        .expect("Given an existing writable root, persisting a record should always succeed");
    let result = repository
        .get(key)
        .await
        .expect("Given a record persisted under this key, reading it back should always succeed");

    assert_eq!(result, expected_result);
}
