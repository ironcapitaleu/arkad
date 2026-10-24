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

use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use uuid::Uuid;

use storage::implementations::repository::filesystem::{
    Document, DocumentKey, DocumentMetadata, FilesystemRepository, Producer,
};
use storage::{ReadRepository, WriteRepository};

/// The document kinds of this test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestDocumentKind {
    RoundTrip,
}

impl fmt::Display for TestDocumentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RoundTrip => write!(f, "round_trip"),
        }
    }
}

/// The metadata fields that only a [`RoundTripDocument`] has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RoundTripContext {
    url: String,
}

/// A document kind that exists only for this test.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RoundTripDocument {
    key: DocumentKey,
    bytes: Vec<u8>,
    metadata: DocumentMetadata<RoundTripContext>,
}

impl Document for RoundTripDocument {
    type Kind = TestDocumentKind;
    const KIND: TestDocumentKind = TestDocumentKind::RoundTrip;
    const KIND_VERSION: u32 = 1;
    type Context = RoundTripContext;

    fn key(&self) -> &DocumentKey {
        &self.key
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn metadata(&self) -> &DocumentMetadata<RoundTripContext> {
        &self.metadata
    }

    fn from_parts(
        key: DocumentKey,
        bytes: Vec<u8>,
        metadata: DocumentMetadata<RoundTripContext>,
    ) -> Self {
        Self {
            key,
            bytes,
            metadata,
        }
    }
}

fn round_trip_document() -> RoundTripDocument {
    let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
        .expect("Given a valid relative path, the key should always build");
    let produced_at = "2026-09-22T20:05:30Z"
        .parse::<DateTime<Utc>>()
        .expect("Given a hardcoded RFC 3339 timestamp, parsing it should always succeed");
    let metadata = DocumentMetadata::builder()
        .document_type::<RoundTripDocument>()
        .media_type("application/json")
        .content(
            41,
            "9d7e13ec27ff57b43422959a9ae627fd5bd65e67b54af2857e9c454a9262376b",
        )
        .produced_at(produced_at)
        .run(
            Uuid::from_u128(0x0b6f_3c1e_5d2a_4f7b_9c3e_8a1d_2b4c_6e8f),
            Producer::new("Round Trip", "0.1.0", "63c1f59"),
        )
        .context(RoundTripContext {
            url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
        })
        .build();

    RoundTripDocument::from_parts(
        key,
        br#"{"cik":320193, "n":1.50, "dup":1,"dup":2}"#.to_vec(),
        metadata,
    )
}

#[tokio::test]
async fn should_return_the_same_record_when_reading_back_a_persisted_record_by_key() {
    let root = TempDir::new().expect(
        "Given a writable system temp directory, creating a directory in it should always succeed",
    );
    let repository = FilesystemRepository::builder()
        .read_write()
        .document_type::<RoundTripDocument>()
        .root(root.path())
        .build();
    let record = round_trip_document();
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
