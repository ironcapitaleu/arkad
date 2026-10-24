//! # Sample Document
//!
//! Provides [`SampleDocument`], a document kind that exists only for tests, and fixed metadata
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

/// The document kinds that exist only for tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SampleDocumentKind {
    /// The kind of a [`SampleDocument`].
    SampleDocument,
}

impl fmt::Display for SampleDocumentKind {
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

/// A document kind that exists only for tests.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleDocument {
    key: DocumentKey,
    bytes: Vec<u8>,
    metadata: DocumentMetadata<SampleContext>,
}

impl Document for SampleDocument {
    type Kind = SampleDocumentKind;
    const KIND: SampleDocumentKind = SampleDocumentKind::SampleDocument;
    const KIND_VERSION: u32 = 1;
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
    let produced_at = "2026-09-22T20:05:30Z"
        .parse::<DateTime<Utc>>()
        .expect("Given a hardcoded RFC 3339 timestamp, parsing it should always succeed");
    let input_key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
        .expect("Given a valid relative path, the key should always build");
    DocumentMetadata::builder()
        .document_type::<SampleDocument>()
        .media_type("application/json")
        .content(
            2,
            "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
        )
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
            "73a86c6aedc31f77cac2ea4df5f80f0b73a86c6aedc31f77cac2ea4df5f80f0b",
        )])
        .build()
}
