//! # Filesystem Repository Integration Tests
//!
//! Persists a record through [`FilesystemRepository`] and reads the same record from a real
//! directory on disk.
//!
//! ## External Dependencies
//!
//! - A writable system temp directory. Each test creates its own root there and deletes it at the
//!   end.

use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use uuid::Uuid;

use storage::implementations::repository::filesystem::FilesystemRepository;
use storage::{
    Document, DocumentKey, DocumentMetadata, InputDocument, METADATA_VERSION, Producer,
    ReadRepository, WriteRepository,
};

/// The fields that only a [`RoundTripDocument`] has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RoundTripContext {
    url: String,
}

/// The metadata stored beside a [`RoundTripDocument`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RoundTripMetadata {
    metadata_version: u32,
    kind: String,
    media_type: String,
    bytes: u64,
    sha256: String,
    produced_at: DateTime<Utc>,
    execution_id: Uuid,
    producer: Producer,
    inputs: Vec<InputDocument>,
    context: RoundTripContext,
}

impl DocumentMetadata for RoundTripMetadata {
    type Context = RoundTripContext;

    const KIND: &'static str = "round_trip.v1";

    fn metadata_version(&self) -> u32 {
        self.metadata_version
    }

    fn kind(&self) -> &str {
        &self.kind
    }

    fn media_type(&self) -> &str {
        &self.media_type
    }

    fn bytes(&self) -> u64 {
        self.bytes
    }

    fn sha256(&self) -> &str {
        &self.sha256
    }

    fn produced_at(&self) -> DateTime<Utc> {
        self.produced_at
    }

    fn execution_id(&self) -> Uuid {
        self.execution_id
    }

    fn producer(&self) -> &Producer {
        &self.producer
    }

    fn inputs(&self) -> &[InputDocument] {
        &self.inputs
    }

    fn context(&self) -> &Self::Context {
        &self.context
    }
}

/// A document kind that exists only for this test.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RoundTripDocument {
    key: DocumentKey,
    bytes: Vec<u8>,
    metadata: RoundTripMetadata,
}

impl Document for RoundTripDocument {
    type Metadata = RoundTripMetadata;

    fn key(&self) -> &DocumentKey {
        &self.key
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn metadata(&self) -> &Self::Metadata {
        &self.metadata
    }

    fn from_parts(key: DocumentKey, bytes: Vec<u8>, metadata: Self::Metadata) -> Self {
        Self {
            key,
            bytes,
            metadata,
        }
    }
}

fn sample_round_trip_document() -> RoundTripDocument {
    let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
        .expect("Given a valid relative path, the key should always build");
    let produced_at = "2026-09-22T20:05:30Z"
        .parse::<DateTime<Utc>>()
        .expect("Given a hardcoded RFC 3339 timestamp, parsing it should always succeed");
    let metadata = RoundTripMetadata {
        metadata_version: METADATA_VERSION,
        kind: RoundTripMetadata::KIND.to_owned(),
        media_type: "application/json".to_owned(),
        bytes: 41,
        sha256: "9d7e13ec27ff57b43422959a9ae627fd5bd65e67b54af2857e9c454a9262376b".to_owned(),
        produced_at,
        execution_id: Uuid::from_u128(0x0b6f_3c1e_5d2a_4f7b_9c3e_8a1d_2b4c_6e8f),
        producer: Producer::new("Round Trip", "0.1.0", "63c1f59"),
        inputs: Vec::new(),
        context: RoundTripContext {
            url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
        },
    };

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
    let repository = FilesystemRepository::<RoundTripDocument>::new(root.path());
    let record = sample_round_trip_document();
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
