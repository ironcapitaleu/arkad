//! # Document Metadata
//!
//! Provides [`DocumentMetadata`], the metadata stored with every document.
//!
//! It is not the fixed
//! [`filesystem::DocumentMetadata`](crate::implementations::repository::filesystem::DocumentMetadata),
//! which holds the fields of one SEC fetch and is what the filesystem repository stores.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::input_document::InputDocument;
use crate::shared::producer::Producer;
use crate::traits::metadata_context::MetadataContext;

/// Version of the shared fields in [`DocumentMetadata`].
///
/// Raise it when a shared field is removed, renamed, or redefined.
pub const METADATA_VERSION: u32 = 1;

/// Describes one stored document: the fields every kind shares, and the context of its kind.
///
/// The shared fields serialize at the top level of a JSON object. The context serializes as a
/// nested object under `context`. So every kind writes the shared fields under the same
/// names.
///
/// [`DocumentMetadata::new`] sets `kind` from [`MetadataContext::KIND`] and `metadata_version` from
/// [`METADATA_VERSION`]. Deserialized metadata keeps the values it was stored with, and
/// deserializing checks neither of them. Before a reader uses the metadata, it compares
/// [`DocumentMetadata::kind`] with [`MetadataContext::KIND`], and
/// [`DocumentMetadata::metadata_version`] with [`METADATA_VERSION`].
///
/// Nothing checks that the SHA-256 is 64 lowercase hex characters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocumentMetadata<C> {
    metadata_version: u32,
    kind: String,
    media_type: String,
    bytes: u64,
    sha256: String,
    produced_at: DateTime<Utc>,
    execution_id: Uuid,
    producer: Producer,
    inputs: Vec<InputDocument>,
    context: C,
}

impl<C: MetadataContext> DocumentMetadata<C> {
    /// Creates new [`DocumentMetadata`] for a document that the given run and code produced.
    ///
    /// `bytes` is the size of the document and `sha256` the hex-encoded SHA-256 of its bytes.
    /// `inputs` names the documents this one came from, and is empty for a document that came
    /// from none.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chrono::{DateTime, Utc};
    /// use serde::{Deserialize, Serialize};
    /// use storage::{DocumentMetadata, MetadataContext, Producer};
    /// use uuid::Uuid;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct FetchContext {
    ///     url: String,
    /// }
    ///
    /// impl MetadataContext for FetchContext {
    ///     const KIND: &'static str = "raw_fetch.v1";
    /// }
    ///
    /// let metadata = DocumentMetadata::new(
    ///     "application/json",
    ///     2,
    ///     "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
    ///     DateTime::<Utc>::UNIX_EPOCH,
    ///     Uuid::nil(),
    ///     Producer::new("Fetch", "0.1.0", "63c1f59"),
    ///     Vec::new(),
    ///     FetchContext { url: "https://example.com/data.json".to_owned() },
    /// );
    ///
    /// let expected_result = "raw_fetch.v1";
    ///
    /// let result = metadata.kind();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    // Deviation: eight arguments, over clippy's limit of seven. Each argument is a stored field,
    // and a builder for them adds more code than it saves.
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        media_type: impl Into<String>,
        bytes: u64,
        sha256: impl Into<String>,
        produced_at: DateTime<Utc>,
        execution_id: Uuid,
        producer: Producer,
        inputs: Vec<InputDocument>,
        context: C,
    ) -> Self {
        Self {
            metadata_version: METADATA_VERSION,
            kind: C::KIND.to_owned(),
            media_type: media_type.into(),
            bytes,
            sha256: sha256.into(),
            produced_at,
            execution_id,
            producer,
            inputs,
            context,
        }
    }
}

impl<C> DocumentMetadata<C> {
    /// Returns the version of the shared fields stored in the metadata.
    #[must_use]
    pub const fn metadata_version(&self) -> u32 {
        self.metadata_version
    }

    /// Returns the kind stored in the metadata.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Returns the media type of the document bytes, for example `application/json`.
    #[must_use]
    pub fn media_type(&self) -> &str {
        &self.media_type
    }

    /// Returns the size of the document in bytes.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Returns the hex-encoded SHA-256 of the document bytes.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// Returns when the producing step made the document bytes.
    #[must_use]
    pub const fn produced_at(&self) -> DateTime<Utc> {
        self.produced_at
    }

    /// Returns the id of the run that produced the document bytes.
    #[must_use]
    pub const fn execution_id(&self) -> Uuid {
        self.execution_id
    }

    /// Returns the code that wrote the document.
    #[must_use]
    pub const fn producer(&self) -> &Producer {
        &self.producer
    }

    /// Returns the documents this document came from.
    #[must_use]
    pub fn inputs(&self) -> &[InputDocument] {
        &self.inputs
    }

    /// Returns the fields that only this document kind has.
    #[must_use]
    pub const fn context(&self) -> &C {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::document_key::DocumentKey;

    /// The context of a document kind that exists only for these tests.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    struct SampleContext {
        url: String,
    }

    impl MetadataContext for SampleContext {
        const KIND: &'static str = "sample_fetch.v1";
    }

    fn sample_metadata() -> DocumentMetadata<SampleContext> {
        let produced_at = "2026-09-22T20:05:30Z"
            .parse::<DateTime<Utc>>()
            .expect("Given a hardcoded RFC 3339 timestamp, parsing it should always succeed");
        let input_key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");
        DocumentMetadata::new(
            "application/json",
            2,
            "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
            produced_at,
            Uuid::from_u128(0x0b6f_3c1e_5d2a_4f7b_9c3e_8a1d_2b4c_6e8f),
            Producer::new("Sample Step", "0.1.0", "63c1f59"),
            vec![InputDocument::new(
                input_key,
                "73a86c6aedc31f77cac2ea4df5f80f0b73a86c6aedc31f77cac2ea4df5f80f0b",
            )],
            SampleContext {
                url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
            },
        )
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_document_metadata() {
        implements_auto_traits::<DocumentMetadata<SampleContext>>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_document_metadata() {
        implements_value_traits::<DocumentMetadata<SampleContext>>();
    }

    #[test]
    fn should_take_the_kind_from_the_context_when_creating_metadata() {
        let metadata = sample_metadata();

        let expected_result = "sample_fetch.v1";

        let result = metadata.kind();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_current_version_when_creating_metadata() {
        let metadata = sample_metadata();

        let expected_result = METADATA_VERSION;

        let result = metadata.metadata_version();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_shared_fields_at_the_top_level_and_nest_the_context() {
        let metadata = sample_metadata();

        let expected_result = serde_json::json!({
            "metadata_version": 1,
            "kind": "sample_fetch.v1",
            "media_type": "application/json",
            "bytes": 2,
            "sha256": "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
            "produced_at": "2026-09-22T20:05:30Z",
            "execution_id": "0b6f3c1e-5d2a-4f7b-9c3e-8a1d2b4c6e8f",
            "producer": { "name": "Sample Step", "crate_version": "0.1.0", "commit": "63c1f59" },
            "inputs": [{
                "key": "sec/companyfacts/CIK0000320193.json",
                "sha256": "73a86c6aedc31f77cac2ea4df5f80f0b73a86c6aedc31f77cac2ea4df5f80f0b",
            }],
            "context": {
                "url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json",
            },
        });

        let result = serde_json::to_value(metadata)
            .expect("Given metadata of owned fields, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_same_metadata_when_deserializing_serialized_metadata() {
        let json = serde_json::to_string(&sample_metadata())
            .expect("Given metadata of owned fields, serialization should always succeed");

        let expected_result = sample_metadata();

        let result: DocumentMetadata<SampleContext> = serde_json::from_str(&json)
            .expect("Given JSON this type produced, deserialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_stored_version_when_reading_metadata_of_another_version() {
        let mut stored = serde_json::to_value(sample_metadata())
            .expect("Given metadata of owned fields, serialization should always succeed");
        stored["metadata_version"] = 7.into();

        let expected_result = 7;

        let result = serde_json::from_value::<DocumentMetadata<SampleContext>>(stored)
            .expect("Given JSON with every field, deserialization should always succeed")
            .metadata_version();

        assert_eq!(result, expected_result);
    }
}
