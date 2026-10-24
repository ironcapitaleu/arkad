//! # Raw Document Metadata Context
//!
//! Provides [`RawDocumentMetadataContext`], the metadata fields that only a
//! [`RawDocument`](super::RawDocument) has.

use serde::{Deserialize, Serialize};

/// The metadata fields that only a [`RawDocument`](super::RawDocument) has: the request that
/// fetched it.
///
/// The store writes these fields as the nested `context` of the metadata file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RawDocumentMetadataContext {
    /// The URL the fetch requested.
    pub url: String,
    /// The HTTP status the fetch returned.
    pub http_status: u16,
    /// The `User-Agent` the fetch sent.
    pub user_agent: String,
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    fn sample_context() -> RawDocumentMetadataContext {
        RawDocumentMetadataContext {
            url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
            http_status: 200,
            user_agent: "arkad contact@example.com".to_owned(),
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_raw_document_metadata_context() {
        implements_auto_traits::<RawDocumentMetadataContext>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_raw_document_metadata_context() {
        implements_value_traits::<RawDocumentMetadataContext>();
    }

    #[test]
    fn should_serialize_the_three_fields_under_their_names_for_raw_document_metadata_context() {
        let context = sample_context();

        let expected_result = serde_json::json!({
            "url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json",
            "http_status": 200,
            "user_agent": "arkad contact@example.com",
        });

        let result = serde_json::to_value(context)
            .expect("Given a context of owned fields, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_same_context_when_deserializing_a_serialized_context() {
        let json = serde_json::to_string(&sample_context())
            .expect("Given a context of owned fields, serialization should always succeed");

        let expected_result = sample_context();

        let result: RawDocumentMetadataContext = serde_json::from_str(&json)
            .expect("Given JSON this type produced, deserialization should always succeed");

        assert_eq!(result, expected_result);
    }
}
