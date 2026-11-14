//! # Stored Form Index
//!
//! Provides [`StoredFormIndex`], the key, the SHA-256, and the text of one stored form index: a
//! daily index or a full index.

use bytes::Bytes;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use storage::Sha256Digest;
use storage::implementations::repository::filesystem::{DocumentKey, InputDocument};

/// The key, the SHA-256, and the text of one stored form index.
///
/// The text shares the body buffer of the downloaded response, so it holds the bytes exactly as
/// the SEC sent them.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoredFormIndex {
    /// The key the index is stored under.
    pub key: DocumentKey,
    /// The SHA-256 digest of the stored bytes.
    pub sha256: Sha256Digest,
    /// The text of the index, as the SEC sent it.
    pub text: Bytes,
}

impl StoredFormIndex {
    /// Returns the index as an [`InputDocument`], for the metadata of a document made from it.
    #[must_use]
    pub fn input_document(&self) -> InputDocument {
        InputDocument::new(self.key.clone(), self.sha256.clone())
    }
}

/// Serializes the key and the size of the text, without the text.
impl Serialize for StoredFormIndex {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("StoredFormIndex", 2)?;
        state.serialize_field("key", &self.key)?;
        state.serialize_field("size_in_bytes", &self.text.len())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const KEY: &str = "sec/full-index/2026/QTR4/form.idx";

    fn stored_index(text: &'static [u8]) -> StoredFormIndex {
        StoredFormIndex {
            key: DocumentKey::new(KEY)
                .expect("Given a hardcoded valid relative path, the key should always build"),
            sha256: Sha256Digest::of(text),
            text: Bytes::from_static(text),
        }
    }

    #[test]
    fn should_serialize_the_key_and_the_size_without_the_text_for_a_stored_form_index() {
        let index = stored_index(b"Form Type");

        let expected_result = serde_json::json!({ "key": KEY, "size_in_bytes": 9 });

        let result = serde_json::to_value(&index)
            .expect("Given a key and a size, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_key_and_the_sha256_of_the_text_for_the_input_document() {
        let index = stored_index(b"Form Type");

        let expected_result = InputDocument::new(
            DocumentKey::new(KEY)
                .expect("Given a hardcoded valid relative path, the key should always build"),
            Sha256Digest::of(b"Form Type"),
        );

        let result = index.input_document();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_stored_form_index() {
        implements_auto_traits::<StoredFormIndex>();
    }
}
