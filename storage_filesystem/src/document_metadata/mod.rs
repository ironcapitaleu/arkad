//! # Document Metadata
//!
//! Provides [`DocumentMetadata`], the record of where one stored document came from.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Describes the fetch that produced one stored document, in its own JSON file beside it.
///
/// Every field records something only the fetch knows, so none of it is recoverable from the
/// document afterwards. The source API sends no `ETag` and no `Last-Modified`, which leaves
/// [`DocumentMetadata::sha256`] as the only way to tell whether a refetch returned new bytes.
///
/// [`DocumentMetadata`] is a plain record. Nothing checks that [`DocumentMetadata::bytes`]
/// matches the document's length or that [`DocumentMetadata::sha256`] is a well-formed digest.
///
/// # Examples
///
/// ```rust
/// use chrono::{TimeZone, Utc};
/// use storage_filesystem::DocumentMetadata;
///
/// let metadata = DocumentMetadata {
///     metadata_version: 1,
///     url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
///     fetched_at: Utc
///         .with_ymd_and_hms(2026, 9, 22, 20, 5, 30)
///         .single()
///         .expect("Given a hardcoded valid timestamp, the conversion should always succeed"),
///     http_status: 200,
///     user_agent: "arkad contact@example.com".to_owned(),
///     bytes: 2,
///     sha256: "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a".to_owned(),
/// };
///
/// let expected_result = "2026-09-22T20:05:30Z";
///
/// let json = serde_json::to_value(&metadata)
///     .expect("Given a metadata value of owned fields, serialization should always succeed");
/// let result = json["fetched_at"]
///     .as_str()
///     .expect("Given a serialized timestamp, the field should always hold a string");
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Version of this metadata format, so a later change stays readable.
    pub metadata_version: u32,
    /// The URL the document was fetched from.
    pub url: String,
    /// When the fetch that produced these exact bytes completed.
    pub fetched_at: DateTime<Utc>,
    /// HTTP status the fetch returned.
    pub http_status: u16,
    /// The `User-Agent` the fetch sent.
    pub user_agent: String,
    /// Size of the document in bytes.
    pub bytes: u64,
    /// Hex-encoded SHA-256 of the document, used to detect a changed refetch.
    pub sha256: String,
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    use super::*;

    fn sample_metadata() -> DocumentMetadata {
        DocumentMetadata {
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
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_document_metadata() {
        implements_auto_traits::<DocumentMetadata>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_document_metadata() {
        implements_send::<DocumentMetadata>();
    }

    #[test]
    const fn should_implement_sync_for_document_metadata() {
        implements_sync::<DocumentMetadata>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_document_metadata() {
        implements_debug::<DocumentMetadata>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_document_metadata() {
        implements_clone::<DocumentMetadata>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_document_metadata() {
        implements_hash::<DocumentMetadata>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_document_metadata() {
        implements_eq::<DocumentMetadata>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_document_metadata() {
        implements_ord::<DocumentMetadata>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_document_metadata() {
        implements_sized::<DocumentMetadata>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_document_metadata() {
        implements_partial_eq::<DocumentMetadata>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_document_metadata() {
        implements_partial_ord::<DocumentMetadata>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_document_metadata() {
        implements_unpin::<DocumentMetadata>();
    }

    #[test]
    fn should_round_trip_through_json_when_serializing_and_deserializing() {
        let expected_result = sample_metadata();
        let json = serde_json::to_string(&expected_result)
            .expect("Given a metadata value of owned fields, serialization should always succeed");

        let result: DocumentMetadata = serde_json::from_str(&json)
            .expect("Given JSON this type produced, deserialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_timestamp_as_rfc_3339_when_writing_json() {
        let json = serde_json::to_value(sample_metadata())
            .expect("Given a metadata value of owned fields, serialization should always succeed");

        let expected_result = "2026-09-22T20:05:30Z";

        let result = json["fetched_at"]
            .as_str()
            .expect("Given a serialized timestamp, the field should always hold a string")
            .to_owned();

        assert_eq!(result, expected_result);
    }
}
