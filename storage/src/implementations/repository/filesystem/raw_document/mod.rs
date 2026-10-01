//! # Raw Document
//!
//! Provides [`RawDocument`], one document and its metadata as the store reads and writes them.

use super::document_key::DocumentKey;
use super::document_metadata::DocumentMetadata;

/// One document, the key naming it, and the metadata of the fetch that produced it.
///
/// Holds the bytes exactly as the source returned them. A parse loses whitespace, number
/// formatting, and duplicate keys, so a parsed document is no longer what arrived.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawDocument {
    key: DocumentKey,
    bytes: Vec<u8>,
    metadata: DocumentMetadata,
}

impl RawDocument {
    /// Creates a new [`RawDocument`] from its key, its bytes, and its metadata.
    ///
    /// The constructor accepts any key, bytes, and metadata. Nothing checks that the metadata
    /// describes these bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use chrono::{TimeZone, Utc};
    /// # use storage::implementations::repository::filesystem::{DocumentKey, DocumentMetadata, RawDocument};
    /// # let metadata = DocumentMetadata {
    /// #     metadata_version: 1,
    /// #     url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
    /// #     fetched_at: Utc.with_ymd_and_hms(2026, 9, 22, 20, 5, 30).single().expect("Given a hardcoded valid timestamp, the conversion should always succeed"),
    /// #     http_status: 200,
    /// #     user_agent: "arkad contact@example.com".to_owned(),
    /// #     bytes: 2,
    /// #     sha256: "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a".to_owned(),
    /// # };
    /// let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
    ///     .expect("Given a valid relative path, the key should always build");
    ///
    /// let document = RawDocument::new(key, b"{}".to_vec(), metadata);
    ///
    /// let expected_result = b"{}";
    ///
    /// let result = document.bytes();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(key: DocumentKey, bytes: Vec<u8>, metadata: DocumentMetadata) -> Self {
        Self {
            key,
            bytes,
            metadata,
        }
    }

    /// Returns the key naming this document.
    #[must_use]
    pub const fn key(&self) -> &DocumentKey {
        &self.key
    }

    /// Returns the document exactly as the source returned it.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the metadata of the fetch that produced these bytes.
    #[must_use]
    pub const fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use chrono::{TimeZone, Utc};
    use pretty_assertions::assert_eq;

    use super::*;

    fn sample_document() -> RawDocument {
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");
        let metadata = DocumentMetadata {
            metadata_version: 1,
            url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
            fetched_at: Utc
                .with_ymd_and_hms(2026, 9, 22, 20, 5, 30)
                .single()
                .expect("Given a hardcoded valid timestamp, the conversion should always succeed"),
            http_status: 200,
            user_agent: "arkad contact@example.com".to_owned(),
            bytes: 2,
            sha256: "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a".to_owned(),
        };

        RawDocument::new(key, b"{}".to_vec(), metadata)
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_raw_document() {
        implements_auto_traits::<RawDocument>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_raw_document() {
        implements_send::<RawDocument>();
    }

    #[test]
    const fn should_implement_sync_for_raw_document() {
        implements_sync::<RawDocument>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_raw_document() {
        implements_debug::<RawDocument>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_raw_document() {
        implements_clone::<RawDocument>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_raw_document() {
        implements_hash::<RawDocument>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_raw_document() {
        implements_eq::<RawDocument>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_raw_document() {
        implements_ord::<RawDocument>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_raw_document() {
        implements_sized::<RawDocument>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_raw_document() {
        implements_partial_eq::<RawDocument>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_raw_document() {
        implements_partial_ord::<RawDocument>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_raw_document() {
        implements_unpin::<RawDocument>();
    }

    #[test]
    fn should_return_the_key_it_was_built_with_when_reading_the_key() {
        let expected_result = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        let result = sample_document().key().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_bytes_unchanged_when_reading_the_document() {
        let expected_result = b"{}";

        let document = sample_document();
        let result = document.bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_metadata_it_was_built_with_when_reading_the_metadata() {
        let expected_result = 200;

        let result = sample_document().metadata().http_status;

        assert_eq!(result, expected_result);
    }
}
