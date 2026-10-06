//! # Sample Raw Document
//!
//! Provides [`sample_raw_document`], a [`RawDocument`] with fixed bytes and metadata.

use chrono::{DateTime, Utc};

use crate::implementations::repository::filesystem::{DocumentKey, DocumentMetadata, RawDocument};

/// Returns a [`RawDocument`] holding `{}` under the key `sec/companyfacts/CIK0000320193.json`.
///
/// # Panics
///
/// Panics if the hardcoded key or the hardcoded timestamp is invalid. Both are valid values.
#[must_use]
pub fn sample_raw_document() -> RawDocument {
    let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
        .expect("Given a valid relative path, the key should always build");
    let fetched_at = "2026-09-22T20:05:30Z"
        .parse::<DateTime<Utc>>()
        .expect("Given a hardcoded RFC 3339 timestamp, parsing it should always succeed");
    let metadata = DocumentMetadata {
        metadata_version: 1,
        url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
        fetched_at,
        http_status: 200,
        user_agent: "arkad contact@example.com".to_owned(),
        bytes: 2,
        sha256: "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a".to_owned(),
    };

    RawDocument::new(key, b"{}".to_vec(), metadata)
}
