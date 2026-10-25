//! # Sample Document
//!
//! Provides [`SampleDocument`], a document type that exists only for tests, and fixed metadata
//! of it.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::implementations::repository::filesystem::shared::document_key::DocumentKey;
use crate::implementations::repository::filesystem::shared::document_metadata::{
    DocumentMetadata, InputDocument, Producer,
};
use crate::implementations::repository::filesystem::traits::document::Document;

/// The document types that exist only for tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SampleDocumentType {
    /// The document type of a [`SampleDocument`].
    SampleDocument,
}

impl fmt::Display for SampleDocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SampleDocument => write!(f, "sample_document"),
        }
    }
}

/// The metadata fields that only a [`SampleDocument`] has.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SampleContext {
    /// Where the sample bytes came from.
    pub url: String,
}

/// A document type that exists only for tests.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleDocument {
    key: DocumentKey,
    bytes: Vec<u8>,
    metadata: DocumentMetadata<SampleContext>,
}

impl Document for SampleDocument {
    type DocumentType = SampleDocumentType;
    const DOCUMENT_TYPE: SampleDocumentType = SampleDocumentType::SampleDocument;
    const DOCUMENT_TYPE_VERSION: u32 = 1;
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

/// Returns fixed metadata of a [`SampleDocument`].
///
/// # Panics
///
/// Panics if the hardcoded timestamp or key is invalid. Both are valid values.
#[must_use]
pub fn sample_metadata() -> DocumentMetadata<SampleContext> {
    sample_metadata_of(b"{}", "2026-09-22T20:05:30Z")
}

/// Returns the metadata of [`sample_metadata`] for the given bytes, produced at the given RFC 3339
/// timestamp.
///
/// # Panics
///
/// Panics if the timestamp is not valid RFC 3339. Every caller passes a hardcoded valid one.
#[must_use]
pub fn sample_metadata_of(bytes: &[u8], produced_at: &str) -> DocumentMetadata<SampleContext> {
    let produced_at = produced_at
        .parse::<DateTime<Utc>>()
        .expect("Given a hardcoded RFC 3339 timestamp, parsing it should always succeed");
    let input_key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
        .expect("Given a valid relative path, the key should always build");
    DocumentMetadata::builder()
        .document_type::<SampleDocument>()
        .media_type("application/json")
        .content(bytes)
        .produced_at(produced_at)
        .run(
            Uuid::from_u128(0x0b6f_3c1e_5d2a_4f7b_9c3e_8a1d_2b4c_6e8f),
            Producer::new("Sample State", "0.1.0", "63c1f59"),
        )
        .context(SampleContext {
            url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
        })
        .inputs(vec![InputDocument::new(
            input_key,
            "73a86c6aedc31f77cac2ea4df5f80f0b73a86c6aedc31f77cac2ea4df5f80f0b"
                .parse()
                .expect(
                    "Given 64 hardcoded lowercase hex characters, parsing should always succeed",
                ),
        )])
        .build()
}

/// Returns a [`SampleDocument`] holding `{}` under the key `sec/companyfacts/CIK0000320193.json`.
///
/// # Panics
///
/// Panics if the hardcoded key is invalid. It is a valid value.
#[must_use]
pub fn sample_document() -> SampleDocument {
    let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
        .expect("Given a valid relative path, the key should always build");
    SampleDocument::from_parts(key, b"{}".to_vec(), sample_metadata())
}

/// Returns a [`SampleDocument`] under the key of [`sample_document`] that holds the given bytes,
/// with metadata produced at the given RFC 3339 timestamp.
///
/// # Panics
///
/// Panics if the hardcoded key or the timestamp is invalid. Every caller passes valid values.
#[must_use]
pub fn sample_document_holding(bytes: &[u8], produced_at: &str) -> SampleDocument {
    let key = sample_document().key().clone();
    SampleDocument::from_parts(key, bytes.to_vec(), sample_metadata_of(bytes, produced_at))
}
