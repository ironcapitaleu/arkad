//! # Sample Document
//!
//! Provides [`SampleDocument`], a document kind that exists only for tests, and fixed values of it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::document_key::DocumentKey;
use crate::shared::document_metadata::DocumentMetadata;
use crate::shared::producer::Producer;
use crate::traits::document::Document;
use crate::traits::metadata_context::MetadataContext;

/// The metadata fields that only a [`SampleDocument`] has.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SampleContext {
    /// Where the sample bytes came from.
    pub source: String,
}

impl MetadataContext for SampleContext {
    const KIND: &'static str = "sample_document.v1";
}

/// A document kind that exists only for tests.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleDocument {
    key: DocumentKey,
    bytes: Vec<u8>,
    metadata: DocumentMetadata<SampleContext>,
}

impl Document for SampleDocument {
    type Context = SampleContext;

    fn key(&self) -> &DocumentKey {
        &self.key
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn metadata(&self) -> &DocumentMetadata<SampleContext> {
        &self.metadata
    }

    fn from_parts(
        key: DocumentKey,
        bytes: Vec<u8>,
        metadata: DocumentMetadata<SampleContext>,
    ) -> Self {
        Self {
            key,
            bytes,
            metadata,
        }
    }
}

/// Returns the metadata of [`sample_document`], produced at the given RFC 3339 timestamp.
///
/// # Panics
///
/// Panics if the timestamp is not valid RFC 3339. Every caller passes a hardcoded valid one.
#[must_use]
pub fn sample_metadata_produced_at(produced_at: &str) -> DocumentMetadata<SampleContext> {
    let produced_at = produced_at
        .parse::<DateTime<Utc>>()
        .expect("Given a hardcoded RFC 3339 timestamp, parsing it should always succeed");
    DocumentMetadata::new(
        "application/json",
        2,
        "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
        produced_at,
        Uuid::from_u128(0x0b6f_3c1e_5d2a_4f7b_9c3e_8a1d_2b4c_6e8f),
        Producer::new("Sample Step", "0.1.0", "63c1f59"),
        Vec::new(),
        SampleContext {
            source: "a hardcoded test value".to_owned(),
        },
    )
}

/// Returns a [`SampleDocument`] holding `{}` under the key `sec/companyfacts/CIK0000320193.json`.
///
/// # Panics
///
/// Panics if the hardcoded key or timestamp is invalid. Both are valid values.
#[must_use]
pub fn sample_document() -> SampleDocument {
    let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
        .expect("Given a valid relative path, the key should always build");
    SampleDocument::from_parts(
        key,
        b"{}".to_vec(),
        sample_metadata_produced_at("2026-09-22T20:05:30Z"),
    )
}
