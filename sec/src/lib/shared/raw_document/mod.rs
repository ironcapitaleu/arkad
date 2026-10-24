//! # Raw Document
//!
//! Provides [`RawDocument`], one SEC file as the store keeps it, with its metadata.
//!
//! A raw document holds the bytes exactly as the SEC returned them. A parse loses whitespace,
//! number formatting, and duplicate keys, so a parsed body is no longer what arrived.
//!
//! ## Modules
//!
//! - [`raw_document_metadata_context`]: The [`RawDocumentMetadataContext`] with the fields that
//!   only a raw document has.
//! - [`raw_source`]: The [`RawSource`] trait that names the SEC file, and one marker per file,
//!   such as [`CompanyFacts`].

use std::marker::PhantomData;

use storage::implementations::repository::filesystem::{Document, DocumentKey, DocumentMetadata};

use crate::shared::document_kind::SecDocumentKind;

pub mod raw_document_metadata_context;
pub mod raw_source;

pub use raw_document_metadata_context::RawDocumentMetadataContext;
pub use raw_source::{CompanyFacts, RawSource};

/// One SEC file of the source `S`, the key naming it, and the metadata of the fetch that produced
/// it.
///
/// `S` sets the document's kind, so `RawDocument<CompanyFacts>` and a raw document of another file
/// are different types. A repository of one never accepts the other.
///
/// # Examples
///
/// ```rust
/// use chrono::{DateTime, Utc};
/// use sec::shared::raw_document::{CompanyFacts, RawDocument, RawDocumentMetadataContext};
/// use storage::implementations::repository::filesystem::{
///     Document, DocumentKey, DocumentMetadata, Producer,
/// };
/// use uuid::Uuid;
///
/// let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
///     .expect("Given a valid relative path, the key should always build");
/// let metadata = DocumentMetadata::builder()
///     .document_type::<RawDocument<CompanyFacts>>()
///     .media_type("application/json")
///     .content(2, "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a")
///     .produced_at(DateTime::<Utc>::UNIX_EPOCH)
///     .run(Uuid::nil(), Producer::new("Store Company Facts", "0.1.0", "63c1f59"))
///     .context(RawDocumentMetadataContext {
///         url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
///         http_status: 200,
///         user_agent: "arkad contact@example.com".to_owned(),
///     })
///     .build();
///
/// let document = RawDocument::<CompanyFacts>::from_parts(key, b"{}".to_vec(), metadata);
///
/// let expected_result = "raw_company_facts";
///
/// let result = document.metadata().kind();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawDocument<S: RawSource> {
    key: DocumentKey,
    bytes: Vec<u8>,
    metadata: DocumentMetadata<RawDocumentMetadataContext>,
    // `fn() -> S` instead of `S`: the document holds no `S`, so it stays `Send`, `Sync`, and
    // `Unpin` for every `S`.
    source: PhantomData<fn() -> S>,
}

impl<S: RawSource> Document for RawDocument<S> {
    type Kind = SecDocumentKind;
    const KIND: SecDocumentKind = S::KIND;
    const KIND_VERSION: u32 = S::KIND_VERSION;
    type Context = RawDocumentMetadataContext;

    fn key(&self) -> &DocumentKey {
        &self.key
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn metadata(&self) -> &DocumentMetadata<RawDocumentMetadataContext> {
        &self.metadata
    }

    fn from_parts(
        key: DocumentKey,
        bytes: Vec<u8>,
        metadata: DocumentMetadata<RawDocumentMetadataContext>,
    ) -> Self {
        Self {
            key,
            bytes,
            metadata,
            source: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use chrono::{DateTime, Utc};
    use pretty_assertions::assert_eq;
    use storage::ReadWriteRepository;
    use storage::implementations::repository::filesystem::{
        FilesystemRepository, Producer, ReadWrite,
    };
    use uuid::Uuid;

    use super::*;

    fn sample_raw_document() -> RawDocument<CompanyFacts> {
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");
        let metadata = DocumentMetadata::builder()
            .document_type::<RawDocument<CompanyFacts>>()
            .media_type("application/json")
            .content(
                2,
                "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
            )
            .produced_at(DateTime::<Utc>::UNIX_EPOCH)
            .run(
                Uuid::nil(),
                Producer::new("Store Company Facts", "0.1.0", "63c1f59"),
            )
            .context(RawDocumentMetadataContext {
                url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
                http_status: 200,
                user_agent: "arkad contact@example.com".to_owned(),
            })
            .build();
        RawDocument::from_parts(key, b"{}".to_vec(), metadata)
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_raw_document_of_company_facts() {
        implements_auto_traits::<RawDocument<CompanyFacts>>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_raw_document_of_company_facts() {
        implements_value_traits::<RawDocument<CompanyFacts>>();
    }

    const fn implements_document<T: Document>() {}
    #[test]
    const fn should_implement_document_for_raw_document_of_company_facts() {
        implements_document::<RawDocument<CompanyFacts>>();
    }

    const fn implements_read_write_repository<T: ReadWriteRepository>() {}
    #[test]
    const fn should_implement_read_write_repository_for_a_filesystem_repository_of_raw_document() {
        implements_read_write_repository::<
            FilesystemRepository<ReadWrite, RawDocument<CompanyFacts>>,
        >();
    }

    #[test]
    fn should_store_the_raw_company_facts_kind_when_building_its_metadata() {
        let document = sample_raw_document();

        let expected_result = "raw_company_facts";

        let result = document.metadata().kind();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_bytes_when_building_a_raw_document_from_its_parts() {
        let document = sample_raw_document();

        let expected_result: &[u8] = b"{}";

        let result = document.bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_key_when_building_a_raw_document_from_its_parts() {
        let document = sample_raw_document();

        let expected_result = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        let result = document.key().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_context_when_building_a_raw_document_from_its_parts() {
        let document = sample_raw_document();

        let expected_result = RawDocumentMetadataContext {
            url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json".to_owned(),
            http_status: 200,
            user_agent: "arkad contact@example.com".to_owned(),
        };

        let result = document.metadata().context().clone();

        assert_eq!(result, expected_result);
    }
}
