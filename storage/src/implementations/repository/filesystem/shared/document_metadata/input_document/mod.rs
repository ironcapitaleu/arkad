//! # Input Document
//!
//! Provides [`InputDocument`], the pointer from a stored document to a document it came from.

use serde::{Deserialize, Serialize};

use crate::implementations::repository::filesystem::shared::document_key::DocumentKey;
use crate::shared::Sha256Digest;

/// Points at one document that a stored document came from.
///
/// Holds the input's key and the SHA-256 of the input's bytes at the time they were read. If the
/// key holds other bytes when it is read again, their SHA-256 differs from the one stored here.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InputDocument {
    key: DocumentKey,
    sha256: Sha256Digest,
}

impl InputDocument {
    /// Creates a new [`InputDocument`] from the input's key and the SHA-256 digest of its bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::implementations::repository::filesystem::{DocumentKey, InputDocument};
    ///
    /// let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
    ///     .expect("Given a valid relative path, the key should always build");
    ///
    /// let input = InputDocument::new(
    ///     key.clone(),
    ///     storage::shared::Sha256Digest::of(b"{}"),
    /// );
    ///
    /// let expected_result = &key;
    ///
    /// let result = input.key();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(key: DocumentKey, sha256: Sha256Digest) -> Self {
        Self { key, sha256 }
    }

    /// Returns the key of the input document.
    #[must_use]
    pub const fn key(&self) -> &DocumentKey {
        &self.key
    }

    /// Returns the SHA-256 digest of the input's bytes when they were read.
    #[must_use]
    pub const fn sha256(&self) -> &Sha256Digest {
        &self.sha256
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    fn sample_input_document() -> InputDocument {
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");
        InputDocument::new(key, Sha256Digest::of(b"{}"))
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_input_document() {
        implements_auto_traits::<InputDocument>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_input_document() {
        implements_send::<InputDocument>();
    }

    #[test]
    const fn should_implement_sync_for_input_document() {
        implements_sync::<InputDocument>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_input_document() {
        implements_debug::<InputDocument>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_input_document() {
        implements_clone::<InputDocument>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_input_document() {
        implements_hash::<InputDocument>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_input_document() {
        implements_eq::<InputDocument>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_input_document() {
        implements_ord::<InputDocument>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_input_document() {
        implements_sized::<InputDocument>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_input_document() {
        implements_partial_eq::<InputDocument>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_input_document() {
        implements_partial_ord::<InputDocument>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_input_document() {
        implements_unpin::<InputDocument>();
    }

    #[test]
    fn should_return_the_key_when_building_an_input_document() {
        let input = sample_input_document();

        let expected_result = &DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        let result = input.key();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_sha256_when_building_an_input_document() {
        let input = sample_input_document();

        let expected_result = &Sha256Digest::of(b"{}");

        let result = input.sha256();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_key_and_the_sha256_when_writing_json() {
        let input = sample_input_document();

        let expected_result = serde_json::json!({
            "key": "sec/companyfacts/CIK0000320193.json",
            "sha256": "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
        });

        let result = serde_json::to_value(input)
            .expect("Given an input holding a built key, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_same_input_when_deserializing_a_serialized_input() {
        let json = serde_json::to_string(&sample_input_document())
            .expect("Given an input holding a built key, serialization should always succeed");

        let expected_result = sample_input_document();

        let result: InputDocument = serde_json::from_str(&json)
            .expect("Given JSON this type produced, deserialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_key_that_escapes_the_root_when_deserializing_an_input_document() {
        let json = r#"{"key":"../escape.json","sha256":"44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"}"#;

        let expected_result =
            "[ContainsParentComponent] Document key '../escape.json' contains a parent component";

        let result = serde_json::from_str::<InputDocument>(json)
            .expect_err("Given a key with a parent component, deserialization should always fail")
            .to_string();

        assert!(result.starts_with(expected_result));
    }
}
