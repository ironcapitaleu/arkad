//! # Sample Document
//!
//! Provides [`SampleDocument`] and [`SampleMetadata`], a document kind that exists only for tests,
//! and [`sample_document`], one fixed value of it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::document_key::DocumentKey;
use crate::shared::input_document::InputDocument;
use crate::shared::producer::Producer;
use crate::traits::document::Document;
use crate::traits::document_metadata::{DocumentMetadata, METADATA_VERSION};

/// The fields that only a [`SampleDocument`] has.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SampleContext {
    /// Where the sample bytes came from.
    pub source: String,
}

/// The metadata stored beside a [`SampleDocument`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SampleMetadata {
    /// Version of the shared fields.
    pub metadata_version: u32,
    /// Always [`SampleMetadata::KIND`].
    pub kind: String,
    /// Media type of the document bytes.
    pub media_type: String,
    /// Size of the document in bytes.
    pub bytes: u64,
    /// Hex-encoded SHA-256 of the document bytes.
    pub sha256: String,
    /// When the document bytes were made.
    pub produced_at: DateTime<Utc>,
    /// The run that made the document bytes.
    pub execution_id: Uuid,
    /// The code that wrote the document.
    pub producer: Producer,
    /// The documents this one came from.
    pub inputs: Vec<InputDocument>,
    /// The fields that only this kind has.
    pub context: SampleContext,
}

impl DocumentMetadata for SampleMetadata {
    type Context = SampleContext;

    const KIND: &'static str = "sample_document.v1";

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

/// A document kind that exists only for tests.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleDocument {
    /// The key naming the document.
    pub key: DocumentKey,
    /// The document content.
    pub bytes: Vec<u8>,
    /// The metadata stored beside the document.
    pub metadata: SampleMetadata,
}

impl Document for SampleDocument {
    type Metadata = SampleMetadata;

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

/// Returns the [`SampleMetadata`] of [`sample_document`].
///
/// # Panics
///
/// Panics if the hardcoded timestamp is invalid. It is a valid value.
#[must_use]
pub fn sample_metadata() -> SampleMetadata {
    let produced_at = "2026-09-22T20:05:30Z"
        .parse::<DateTime<Utc>>()
        .expect("Given a hardcoded RFC 3339 timestamp, parsing it should always succeed");
    SampleMetadata {
        metadata_version: METADATA_VERSION,
        kind: SampleMetadata::KIND.to_owned(),
        media_type: "application/json".to_owned(),
        bytes: 2,
        sha256: "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a".to_owned(),
        produced_at,
        execution_id: Uuid::from_u128(0x0b6f_3c1e_5d2a_4f7b_9c3e_8a1d_2b4c_6e8f),
        producer: Producer::new("Sample Step", "0.1.0", "63c1f59"),
        inputs: Vec::new(),
        context: SampleContext {
            source: "a hardcoded test value".to_owned(),
        },
    }
}

/// Returns a [`SampleDocument`] holding `{}` under the key `sec/companyfacts/CIK0000320193.json`.
///
/// # Panics
///
/// Panics if the hardcoded key or the hardcoded timestamp is invalid. Both are valid values.
#[must_use]
pub fn sample_document() -> SampleDocument {
    let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
        .expect("Given a valid relative path, the key should always build");
    SampleDocument::from_parts(key, b"{}".to_vec(), sample_metadata())
}
