//! # Builder
//!
//! Provides [`DocumentMetadataBuilder`], which builds a [`DocumentMetadata`] for one document
//! type.
//!
//! Each step changes the builder's type. [`build`](DocumentMetadataBuilder::build) exists only once
//! the builder holds the document type and every required field, so metadata without one of them
//! does not compile. The document type sets `document_type`, `document_type_version`, the
//! context type, and the type of the ingestion mode.

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::implementations::repository::filesystem::shared::document_metadata::DocumentMetadata;
use crate::implementations::repository::filesystem::shared::document_metadata::constants::METADATA_VERSION;
use crate::implementations::repository::filesystem::shared::document_metadata::input_document::InputDocument;
use crate::implementations::repository::filesystem::shared::document_metadata::producer::Producer;
use crate::implementations::repository::filesystem::traits::document::Document;
use crate::shared::Sha256Digest;

/// Marks a builder that holds no document type yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoDocumentType;

/// Marks a builder that holds no media type yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoMediaType;

/// Marks a builder that holds no content size and digest yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoContent;

/// Marks a builder that holds no production time yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoProducedAt;

/// Marks a builder that holds no run yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoRun;

/// Marks a builder that holds no ingestion mode yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoIngestionMode;

/// Marks a builder that holds no context yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoContext;

/// The size and the SHA-256 of one document's bytes, as a builder holds them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Content {
    size_in_bytes: u64,
    sha256: Sha256Digest,
}

/// The run and the code that produced one document, as a builder holds them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Run {
    execution_id: Uuid,
    producer: Producer,
}

/// Builds a [`DocumentMetadata`] for one document type.
///
/// Set the document type with [`document_type`](Self::document_type), and the required fields
/// with [`media_type`](Self::media_type), [`content`](Self::content),
/// [`produced_at`](Self::produced_at), [`run`](Self::run),
/// [`ingestion_mode`](Self::ingestion_mode), and [`context`](Self::context).
/// [`inputs`](Self::inputs) is optional and defaults to no inputs. Then call
/// [`build`](Self::build).
///
/// [`ingestion_mode`](Self::ingestion_mode) takes the document type's
/// [`Document::IngestionMode`], so it comes after [`document_type`](Self::document_type). All
/// other steps come in any order.
///
/// The derived traits require `D` to implement them too. A builder exists only between
/// [`DocumentMetadata::builder`] and [`build`](Self::build), so this matters only for code that
/// compares, orders, or hashes a builder.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentMetadataBuilder<D, M, B, P, R, I, C> {
    document_type: PhantomData<fn() -> D>,
    media_type: M,
    content: B,
    produced_at: P,
    run: R,
    ingestion_mode: I,
    context: C,
    inputs: Vec<InputDocument>,
}

impl
    DocumentMetadataBuilder<
        NoDocumentType,
        NoMediaType,
        NoContent,
        NoProducedAt,
        NoRun,
        NoIngestionMode,
        NoContext,
    >
{
    /// Creates a new [`DocumentMetadataBuilder`] that holds no document type, no field, and no
    /// inputs.
    pub(crate) const fn new() -> Self {
        Self {
            document_type: PhantomData,
            media_type: NoMediaType,
            content: NoContent,
            produced_at: NoProducedAt,
            run: NoRun,
            ingestion_mode: NoIngestionMode,
            context: NoContext,
            inputs: Vec::new(),
        }
    }
}

impl<M, B, P, R, I, C> DocumentMetadataBuilder<NoDocumentType, M, B, P, R, I, C> {
    /// Sets the document type, which sets `document_type`, `document_type_version`, and the context
    /// type.
    #[must_use]
    pub fn document_type<D: Document>(self) -> DocumentMetadataBuilder<D, M, B, P, R, I, C> {
        DocumentMetadataBuilder {
            document_type: PhantomData,
            media_type: self.media_type,
            content: self.content,
            produced_at: self.produced_at,
            run: self.run,
            ingestion_mode: self.ingestion_mode,
            context: self.context,
            inputs: self.inputs,
        }
    }
}

impl<D, B, P, R, I, C> DocumentMetadataBuilder<D, NoMediaType, B, P, R, I, C> {
    /// Sets the media type of the document bytes, for example `application/json`.
    #[must_use]
    pub fn media_type(
        self,
        media_type: impl Into<String>,
    ) -> DocumentMetadataBuilder<D, String, B, P, R, I, C> {
        DocumentMetadataBuilder {
            document_type: self.document_type,
            media_type: media_type.into(),
            content: self.content,
            produced_at: self.produced_at,
            run: self.run,
            ingestion_mode: self.ingestion_mode,
            context: self.context,
            inputs: self.inputs,
        }
    }
}

impl<D, M, P, R, I, C> DocumentMetadataBuilder<D, M, NoContent, P, R, I, C> {
    /// Sets the size and the SHA-256 digest of the document from its bytes.
    ///
    /// The record must hold the same bytes. `persist` and `get` check that it does.
    #[must_use]
    pub fn content(
        self,
        bytes: impl AsRef<[u8]>,
    ) -> DocumentMetadataBuilder<D, M, Content, P, R, I, C> {
        let bytes = bytes.as_ref();
        DocumentMetadataBuilder {
            document_type: self.document_type,
            media_type: self.media_type,
            content: Content {
                // A `usize` always fits in a `u64` on the platforms Rust supports.
                size_in_bytes: bytes.len() as u64,
                sha256: Sha256Digest::of(bytes),
            },
            produced_at: self.produced_at,
            run: self.run,
            ingestion_mode: self.ingestion_mode,
            context: self.context,
            inputs: self.inputs,
        }
    }
}

impl<D, M, B, R, I, C> DocumentMetadataBuilder<D, M, B, NoProducedAt, R, I, C> {
    /// Sets when the producing state made the document bytes.
    #[must_use]
    pub fn produced_at(
        self,
        produced_at: DateTime<Utc>,
    ) -> DocumentMetadataBuilder<D, M, B, DateTime<Utc>, R, I, C> {
        DocumentMetadataBuilder {
            document_type: self.document_type,
            media_type: self.media_type,
            content: self.content,
            produced_at,
            run: self.run,
            ingestion_mode: self.ingestion_mode,
            context: self.context,
            inputs: self.inputs,
        }
    }
}

impl<D, M, B, P, I, C> DocumentMetadataBuilder<D, M, B, P, NoRun, I, C> {
    /// Sets the id of the run and the code that produced the document bytes.
    #[must_use]
    pub fn run(
        self,
        execution_id: Uuid,
        producer: Producer,
    ) -> DocumentMetadataBuilder<D, M, B, P, Run, I, C> {
        DocumentMetadataBuilder {
            document_type: self.document_type,
            media_type: self.media_type,
            content: self.content,
            produced_at: self.produced_at,
            run: Run {
                execution_id,
                producer,
            },
            ingestion_mode: self.ingestion_mode,
            context: self.context,
            inputs: self.inputs,
        }
    }
}

impl<D: Document, M, B, P, R, C> DocumentMetadataBuilder<D, M, B, P, R, NoIngestionMode, C> {
    /// Sets the ingestion mode of the run that produced the document bytes.
    ///
    /// Takes the document type's [`Document::IngestionMode`] and stores its `Display` text, for
    /// example `backfill`.
    ///
    /// # Examples
    ///
    /// A raw string is not a mode of a document type whose mode is an enum, so this does not
    /// compile:
    ///
    /// ```compile_fail,E0308
    /// # use std::fmt;
    /// #
    /// # use storage::implementations::repository::filesystem::{
    /// #     Document, DocumentKey, DocumentMetadata,
    /// # };
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
    /// enum RunMode {
    ///     Backfill,
    /// }
    ///
    /// impl fmt::Display for RunMode {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "backfill")
    ///     }
    /// }
    /// #
    /// # struct Summary {
    /// #     key: DocumentKey,
    /// #     bytes: Vec<u8>,
    /// #     metadata: DocumentMetadata<()>,
    /// # }
    /// #
    /// # impl Document for Summary {
    /// #     type DocumentType = ReportType;
    /// #     type IngestionMode = RunMode;
    /// #     const DOCUMENT_TYPE: ReportType = ReportType::Summary;
    /// #     const DOCUMENT_TYPE_VERSION: u32 = 1;
    /// #     type Context = ();
    /// #     fn key(&self) -> &DocumentKey { &self.key }
    /// #     fn bytes(&self) -> &[u8] { &self.bytes }
    /// #     fn metadata(&self) -> &DocumentMetadata<()> { &self.metadata }
    /// #     fn from_parts(key: DocumentKey, bytes: Vec<u8>, metadata: DocumentMetadata<()>) -> Self {
    /// #         Self { key, bytes, metadata }
    /// #     }
    /// # }
    ///
    /// let builder = DocumentMetadata::builder()
    ///     .document_type::<Summary>()
    ///     .ingestion_mode(&"foo");
    /// ```
    #[must_use]
    pub fn ingestion_mode(
        self,
        ingestion_mode: &D::IngestionMode,
    ) -> DocumentMetadataBuilder<D, M, B, P, R, String, C> {
        DocumentMetadataBuilder {
            document_type: self.document_type,
            media_type: self.media_type,
            content: self.content,
            produced_at: self.produced_at,
            run: self.run,
            ingestion_mode: ingestion_mode.to_string(),
            context: self.context,
            inputs: self.inputs,
        }
    }
}

impl<D, M, B, P, R, I> DocumentMetadataBuilder<D, M, B, P, R, I, NoContext> {
    /// Sets the fields that only this document type has.
    ///
    /// If `C` is not the document type's [`Document::Context`], the builder has no `build` method.
    #[must_use]
    pub fn context<C>(self, context: C) -> DocumentMetadataBuilder<D, M, B, P, R, I, C> {
        DocumentMetadataBuilder {
            document_type: self.document_type,
            media_type: self.media_type,
            content: self.content,
            produced_at: self.produced_at,
            run: self.run,
            ingestion_mode: self.ingestion_mode,
            context,
            inputs: self.inputs,
        }
    }
}

impl<D, M, B, P, R, I, C> DocumentMetadataBuilder<D, M, B, P, R, I, C> {
    /// Sets the documents this document came from, in place of any set before.
    ///
    /// If you skip this step, the metadata holds no inputs.
    #[must_use]
    pub fn inputs(mut self, inputs: Vec<InputDocument>) -> Self {
        self.inputs = inputs;
        self
    }
}

impl<D: Document>
    DocumentMetadataBuilder<D, String, Content, DateTime<Utc>, Run, String, D::Context>
{
    /// Builds a [`DocumentMetadata`] with the fields this builder holds.
    ///
    /// Sets `metadata_version` from [`METADATA_VERSION`], `document_type` from
    /// [`Document::DOCUMENT_TYPE`], and `document_type_version` from
    /// [`Document::DOCUMENT_TYPE_VERSION`].
    ///
    /// # Examples
    ///
    /// A builder without a document type has no `build` method, so this does not compile:
    ///
    /// ```compile_fail,E0599
    /// use chrono::{DateTime, Utc};
    /// use storage::implementations::repository::filesystem::{DocumentMetadata, Producer};
    /// use uuid::Uuid;
    ///
    /// let bytes = b"{}";
    /// let metadata = DocumentMetadata::builder()
    ///     .media_type("application/json")
    ///     .content(bytes)
    ///     .produced_at(DateTime::<Utc>::UNIX_EPOCH)
    ///     .run(Uuid::nil(), Producer::new("Summarize", "0.1.0", "63c1f59"))
    ///     .context(())
    ///     .build();
    /// ```
    ///
    /// A context of another type than the document type's context has no `build` method either:
    ///
    /// ```compile_fail,E0599
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
    /// #     type IngestionMode = &'static str;
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
    ///     .ingestion_mode(&"backfill")
    ///     .context(())
    ///     .build();
    /// ```
    ///
    /// A builder without a run has no `build` method either:
    ///
    /// ```compile_fail,E0599
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
    /// #     type IngestionMode = &'static str;
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
    ///     .ingestion_mode(&"backfill")
    ///     .context(SummaryContext { source: "a test".to_owned() })
    ///     .build();
    /// ```
    ///
    /// A builder without an ingestion mode has no `build` method either:
    ///
    /// ```compile_fail,E0599
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
    /// #     type IngestionMode = &'static str;
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
    /// ```
    #[must_use]
    pub fn build(self) -> DocumentMetadata<D::Context> {
        DocumentMetadata {
            metadata_version: METADATA_VERSION,
            document_type: D::DOCUMENT_TYPE.to_string(),
            document_type_version: D::DOCUMENT_TYPE_VERSION,
            media_type: self.media_type,
            size_in_bytes: self.content.size_in_bytes,
            sha256: self.content.sha256,
            produced_at: self.produced_at,
            execution_id: self.run.execution_id,
            ingestion_mode: self.ingestion_mode,
            producer: self.run.producer,
            inputs: self.inputs,
            context: self.context,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::fixtures::sample_document::{SampleContext, SampleDocument, sample_metadata};

    fn produced_at() -> DateTime<Utc> {
        "2026-09-22T20:05:30Z"
            .parse::<DateTime<Utc>>()
            .expect("Given a hardcoded RFC 3339 timestamp, parsing it should always succeed")
    }

    fn sample_context() -> SampleContext {
        SampleContext {
            url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
        }
    }

    fn sample_run() -> (Uuid, Producer) {
        (
            Uuid::from_u128(0x0b6f_3c1e_5d2a_4f7b_9c3e_8a1d_2b4c_6e8f),
            Producer::new("Sample State", "0.1.0", "63c1f59"),
        )
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_document_metadata_builder() {
        implements_auto_traits::<
            DocumentMetadataBuilder<
                NoDocumentType,
                NoMediaType,
                NoContent,
                NoProducedAt,
                NoRun,
                NoIngestionMode,
                NoContext,
            >,
        >();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_document_metadata_builder() {
        implements_value_traits::<
            DocumentMetadataBuilder<
                NoDocumentType,
                NoMediaType,
                NoContent,
                NoProducedAt,
                NoRun,
                NoIngestionMode,
                NoContext,
            >,
        >();
    }

    #[test]
    fn should_hold_no_field_and_no_inputs_when_starting_a_builder() {
        let expected_result = DocumentMetadataBuilder {
            document_type: PhantomData,
            media_type: NoMediaType,
            content: NoContent,
            produced_at: NoProducedAt,
            run: NoRun,
            ingestion_mode: NoIngestionMode,
            context: NoContext,
            inputs: Vec::new(),
        };

        let result = DocumentMetadata::builder();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_sample_metadata_when_setting_every_field() {
        let input = sample_metadata().inputs().to_vec();
        let (execution_id, producer) = sample_run();

        let expected_result = sample_metadata();

        let result = DocumentMetadata::builder()
            .document_type::<SampleDocument>()
            .media_type("application/json")
            .content(b"{}")
            .produced_at(produced_at())
            .run(execution_id, producer)
            .ingestion_mode(&"backfill")
            .context(sample_context())
            .inputs(input)
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_same_metadata_when_setting_the_fields_in_reverse_order() {
        let input = sample_metadata().inputs().to_vec();
        let (execution_id, producer) = sample_run();

        let expected_result = sample_metadata();

        let result = DocumentMetadata::builder()
            .inputs(input)
            .context(sample_context())
            .run(execution_id, producer)
            .produced_at(produced_at())
            .content(b"{}")
            .media_type("application/json")
            .document_type::<SampleDocument>()
            .ingestion_mode(&"backfill")
            .build();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_set_the_size_and_the_digest_from_the_bytes_when_setting_the_content() {
        let bytes = b"{}";
        let (execution_id, producer) = sample_run();

        let expected_result = (2, Sha256Digest::of(bytes));

        let metadata = DocumentMetadata::builder()
            .document_type::<SampleDocument>()
            .media_type("application/json")
            .content(bytes)
            .produced_at(produced_at())
            .run(execution_id, producer)
            .ingestion_mode(&"backfill")
            .context(sample_context())
            .build();
        let result = (metadata.size_in_bytes(), metadata.sha256().clone());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_set_size_zero_and_the_empty_digest_when_the_content_is_empty() {
        let bytes = b"";
        let (execution_id, producer) = sample_run();

        let expected_result = (0, Sha256Digest::of(bytes));

        let metadata = DocumentMetadata::builder()
            .document_type::<SampleDocument>()
            .media_type("application/json")
            .content(bytes)
            .produced_at(produced_at())
            .run(execution_id, producer)
            .ingestion_mode(&"backfill")
            .context(sample_context())
            .build();
        let result = (metadata.size_in_bytes(), metadata.sha256().clone());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_no_inputs_when_the_inputs_step_is_left_out() {
        let (execution_id, producer) = sample_run();

        let expected_result: &[InputDocument] = &[];

        let metadata = DocumentMetadata::builder()
            .document_type::<SampleDocument>()
            .media_type("application/json")
            .content(b"{}")
            .produced_at(produced_at())
            .run(execution_id, producer)
            .ingestion_mode(&"backfill")
            .context(sample_context())
            .build();
        let result = metadata.inputs();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_the_ingestion_mode_when_setting_the_ingestion_mode() {
        let (execution_id, producer) = sample_run();

        let expected_result = "backfill";

        let metadata = DocumentMetadata::builder()
            .document_type::<SampleDocument>()
            .media_type("application/json")
            .content(b"{}")
            .produced_at(produced_at())
            .run(execution_id, producer)
            .ingestion_mode(&"backfill")
            .context(sample_context())
            .build();
        let result = metadata.ingestion_mode();

        assert_eq!(result, expected_result);
    }

    #[test]
    const fn should_implement_value_traits_for_no_document_type() {
        implements_value_traits::<NoDocumentType>();
    }

    #[test]
    const fn should_implement_value_traits_for_no_media_type() {
        implements_value_traits::<NoMediaType>();
    }

    #[test]
    const fn should_implement_value_traits_for_no_content() {
        implements_value_traits::<NoContent>();
    }

    #[test]
    const fn should_implement_value_traits_for_no_produced_at() {
        implements_value_traits::<NoProducedAt>();
    }

    #[test]
    const fn should_implement_value_traits_for_no_run() {
        implements_value_traits::<NoRun>();
    }

    #[test]
    const fn should_implement_value_traits_for_no_ingestion_mode() {
        implements_value_traits::<NoIngestionMode>();
    }

    #[test]
    const fn should_implement_value_traits_for_no_context() {
        implements_value_traits::<NoContext>();
    }

    #[test]
    const fn should_implement_value_traits_for_content() {
        implements_value_traits::<Content>();
    }

    #[test]
    const fn should_implement_value_traits_for_run() {
        implements_value_traits::<Run>();
    }
}
