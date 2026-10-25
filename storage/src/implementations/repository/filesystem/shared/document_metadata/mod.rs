//! # Document Metadata
//!
//! Provides [`DocumentMetadata`], the metadata stored beside every document, and its parts.
//!
//! ## Modules
//!
//! - [`builder`]: The [`DocumentMetadataBuilder`] and the markers for its missing steps.
//! - [`constants`]: The version of the shared fields, [`METADATA_VERSION`].
//! - [`input_document`]: The [`InputDocument`] pointing at a document that another document came
//!   from.
//! - [`producer`]: The [`Producer`] naming the code that wrote a document.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::Sha256Digest;

pub mod builder;
pub mod constants;
pub mod input_document;
pub mod producer;

pub use builder::{
    Content, DocumentMetadataBuilder, NoContent, NoContext, NoDocumentType, NoMediaType,
    NoProducedAt, NoRun, Run,
};
pub use constants::METADATA_VERSION;
pub use input_document::InputDocument;
pub use producer::Producer;

/// Describes one stored document: the fields every document type shares, and the context of its
/// document type.
///
/// The shared fields serialize at the top level of a JSON object. The context serializes as a
/// nested object under `context`.
///
/// [`DocumentMetadataBuilder::build`] sets `metadata_version` from [`METADATA_VERSION`], and
/// `document_type` and `document_type_version` from the [`Document`] type it names. The metadata
/// holds no link to that type. Two document types can share one context type, so
/// [`Document::from_parts`] can receive metadata built for another type. Deserializing checks none
/// of the three values either. So a reader compares them with the document type's values before it
/// uses the metadata.
///
/// [`Document`]: crate::implementations::repository::filesystem::Document
/// [`Document::from_parts`]: crate::implementations::repository::filesystem::Document::from_parts
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocumentMetadata<C> {
    metadata_version: u32,
    document_type: String,
    document_type_version: u32,
    media_type: String,
    #[serde(rename = "bytes")]
    size_in_bytes: u64,
    sha256: Sha256Digest,
    produced_at: DateTime<Utc>,
    execution_id: Uuid,
    producer: Producer,
    inputs: Vec<InputDocument>,
    context: C,
}

impl DocumentMetadata<NoContext> {
    /// Starts a [`DocumentMetadataBuilder`] for new [`DocumentMetadata`].
    ///
    /// The impl uses `NoContext` so that a call needs no type annotation. No built metadata has
    /// `NoContext`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::fmt;
    /// #
    /// # use chrono::{DateTime, Utc};
    /// # use serde::{Deserialize, Serialize};
    /// # use storage::implementations::repository::filesystem::{
    /// #     Document, DocumentKey, DocumentMetadata, Producer,
    /// # };
    /// # use uuid::Uuid;
    /// #
    /// # #[derive(Debug, Clone, Copy)]
    /// # enum ReportType {
    /// #     Summary,
    /// # }
    /// #
    /// # impl fmt::Display for ReportType {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "summary")
    /// #     }
    /// # }
    /// #
    /// # #[derive(Serialize, Deserialize)]
    /// # struct SummaryContext {
    /// #     source: String,
    /// # }
    /// #
    /// # struct Summary {
    /// #     key: DocumentKey,
    /// #     bytes: Vec<u8>,
    /// #     metadata: DocumentMetadata<SummaryContext>,
    /// # }
    /// #
    /// # impl Document for Summary {
    /// #     type DocumentType = ReportType;
    /// #     const DOCUMENT_TYPE: ReportType = ReportType::Summary;
    /// #     const DOCUMENT_TYPE_VERSION: u32 = 1;
    /// #     type Context = SummaryContext;
    /// #     fn key(&self) -> &DocumentKey { &self.key }
    /// #     fn bytes(&self) -> &[u8] { &self.bytes }
    /// #     fn metadata(&self) -> &DocumentMetadata<SummaryContext> { &self.metadata }
    /// #     fn from_parts(
    /// #         key: DocumentKey,
    /// #         bytes: Vec<u8>,
    /// #         metadata: DocumentMetadata<SummaryContext>,
    /// #     ) -> Self {
    /// #         Self { key, bytes, metadata }
    /// #     }
    /// # }
    /// #
    /// let bytes = b"{}";
    /// let metadata = DocumentMetadata::builder()
    ///     .document_type::<Summary>()
    ///     .media_type("application/json")
    ///     .content(bytes)
    ///     .produced_at(DateTime::<Utc>::UNIX_EPOCH)
    ///     .run(Uuid::nil(), Producer::new("Summarize", "0.1.0", "63c1f59"))
    ///     .context(SummaryContext { source: "a test".to_owned() })
    ///     .build();
    ///
    /// let expected_result = "summary";
    ///
    /// let result = metadata.document_type();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn builder() -> DocumentMetadataBuilder<
        NoDocumentType,
        NoMediaType,
        NoContent,
        NoProducedAt,
        NoRun,
        NoContext,
    > {
        DocumentMetadataBuilder::new()
    }
}

impl<C> DocumentMetadata<C> {
    /// Returns the version of the shared fields stored in the metadata.
    #[must_use]
    pub const fn metadata_version(&self) -> u32 {
        self.metadata_version
    }

    /// Returns the document type stored in the metadata.
    #[must_use]
    pub fn document_type(&self) -> &str {
        &self.document_type
    }

    /// Returns the version of the context stored in the metadata.
    #[must_use]
    pub const fn document_type_version(&self) -> u32 {
        self.document_type_version
    }

    /// Returns the media type of the document bytes, for example `application/json`.
    #[must_use]
    pub fn media_type(&self) -> &str {
        &self.media_type
    }

    /// Returns the size of the document in bytes.
    #[must_use]
    pub const fn size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }

    /// Returns the SHA-256 digest of the document bytes.
    #[must_use]
    pub const fn sha256(&self) -> &Sha256Digest {
        &self.sha256
    }

    /// Returns when the producing state made the document bytes.
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

    /// Returns the fields that only this document type has.
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
    use crate::tests::fixtures::sample_document::{SampleContext, sample_metadata};

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_document_metadata() {
        implements_auto_traits::<DocumentMetadata<SampleContext>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_document_metadata() {
        implements_debug::<DocumentMetadata<SampleContext>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_document_metadata() {
        implements_clone::<DocumentMetadata<SampleContext>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_document_metadata() {
        implements_partial_eq::<DocumentMetadata<SampleContext>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_document_metadata() {
        implements_eq::<DocumentMetadata<SampleContext>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_document_metadata() {
        implements_partial_ord::<DocumentMetadata<SampleContext>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_document_metadata() {
        implements_ord::<DocumentMetadata<SampleContext>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_document_metadata() {
        implements_hash::<DocumentMetadata<SampleContext>>();
    }

    #[test]
    fn should_take_the_document_type_from_the_document_when_creating_metadata() {
        let metadata = sample_metadata();

        let expected_result = "sample_document";

        let result = metadata.document_type();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_document_type_version_from_the_document_when_creating_metadata() {
        let metadata = sample_metadata();

        let expected_result = 1;

        let result = metadata.document_type_version();

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
    fn should_serialize_the_shared_fields_at_the_top_level_and_nest_the_context_for_document_metadata()
     {
        let metadata = sample_metadata();

        let expected_result = serde_json::json!({
            "metadata_version": 2,
            "document_type": "sample_document",
            "document_type_version": 1,
            "media_type": "application/json",
            "bytes": 2,
            "sha256": "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
            "produced_at": "2026-09-22T20:05:30Z",
            "execution_id": "0b6f3c1e-5d2a-4f7b-9c3e-8a1d2b4c6e8f",
            "producer": { "name": "Sample State", "crate_version": "0.1.0", "commit": "63c1f59" },
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

    #[test]
    fn should_keep_the_stored_document_type_when_reading_metadata_of_another_document_type() {
        let mut stored = serde_json::to_value(sample_metadata())
            .expect("Given metadata of owned fields, serialization should always succeed");
        stored["document_type"] = "another_document_type".into();

        let expected_result = "another_document_type";

        let result = serde_json::from_value::<DocumentMetadata<SampleContext>>(stored)
            .expect("Given JSON with every field, deserialization should always succeed")
            .document_type()
            .to_owned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_stored_document_type_version_when_reading_metadata_of_another_document_type_version()
     {
        let mut stored = serde_json::to_value(sample_metadata())
            .expect("Given metadata of owned fields, serialization should always succeed");
        stored["document_type_version"] = 7.into();

        let expected_result = 7;

        let result = serde_json::from_value::<DocumentMetadata<SampleContext>>(stored)
            .expect("Given JSON with every field, deserialization should always succeed")
            .document_type_version();

        assert_eq!(result, expected_result);
    }
}
