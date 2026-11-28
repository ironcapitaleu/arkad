//! # Sample Source Document
//!
//! Provides [`notes_source`] and [`instance_source`], the raw documents of one sample Notes file and
//! one sample instance.

use storage::Sha256Digest;
use storage::implementations::repository::filesystem::{DocumentKey, IngestionMode};

use crate::datasets::SourceKind;
use crate::shared::source_document::SourceDocument;

/// Returns the [`SourceDocument`] of the Notes file `sec/notes/2025q3_notes.zip`, with the SHA-256
/// of the bytes `notes`.
///
/// # Panics
///
/// Never panics: the key is a hardcoded valid relative path.
#[must_use]
pub fn notes_source() -> SourceDocument {
    SourceDocument {
        kind: SourceKind::Notes,
        key: DocumentKey::new("sec/notes/2025q3_notes.zip")
            .expect("Given a hardcoded valid relative path, the key should always build"),
        sha256: Sha256Digest::of(b"notes"),
        ingestion_mode: IngestionMode::Backfill,
    }
}

/// Returns the [`SourceDocument`] of the instance
/// `sec/filings/320193/000032019326000001/instance.xml`, with the SHA-256 of the bytes `instance`.
///
/// # Panics
///
/// Never panics: the key is a hardcoded valid relative path.
#[must_use]
pub fn instance_source() -> SourceDocument {
    SourceDocument {
        kind: SourceKind::Instance,
        key: DocumentKey::new("sec/filings/320193/000032019326000001/instance.xml")
            .expect("Given a hardcoded valid relative path, the key should always build"),
        sha256: Sha256Digest::of(b"instance"),
        ingestion_mode: IngestionMode::Incremental,
    }
}
