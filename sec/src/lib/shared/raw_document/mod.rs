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

use std::marker::PhantomData;

use storage::implementations::repository::filesystem::{Document, DocumentKey, DocumentMetadata};

use crate::shared::document_kind::SecDocumentKind;
use crate::shared::raw_source::RawSource;

pub mod raw_document_metadata_context;

pub use raw_document_metadata_context::RawDocumentMetadataContext;

/// One SEC file of the source `S`, the key naming it, and the metadata of the fetch that produced
/// it.
///
/// `S` sets the document's kind, so `RawDocument<RawCompanyFacts>` and a raw document of another
/// file are different types. A repository of one rejects the files of the other, as long as each
/// marker names its own kind.
///
/// # Examples
///
/// ```rust
/// use chrono::{DateTime, Utc};
/// use sec::shared::raw_document::{RawDocument, RawDocumentMetadataContext};
/// use sec::shared::raw_source::RawCompanyFacts;
/// use sec::shared::status_code::StatusCode;
/// use sec::shared::user_agent::UserAgent;
/// use storage::implementations::repository::filesystem::{
///     Document, DocumentKey, DocumentMetadata, Producer,
/// };
/// use uuid::Uuid;
///
/// let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
///     .expect("Given a valid relative path, the key should always build");
/// let metadata = DocumentMetadata::builder()
///     .document_type::<RawDocument<RawCompanyFacts>>()
///     .media_type("application/json")
///     .content(2, "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a")
///     .produced_at(DateTime::<Utc>::UNIX_EPOCH)
///     .run(Uuid::nil(), Producer::new("Store Company Facts", "0.1.0", "63c1f59"))
///     .context(RawDocumentMetadataContext {
///         url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
///             .parse()
///             .expect("A hardcoded valid URL string should always parse"),
///         http_status: StatusCode::Ok,
///         user_agent: UserAgent::new("arkad contact@example.com")
///             .expect("A hardcoded SEC-format string should always validate"),
///     })
///     .build();
///
/// let document = RawDocument::<RawCompanyFacts>::from_parts(key, b"{}".to_vec(), metadata);
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
    use crate::shared::raw_source::RawCompanyFacts;
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;
    use crate::shared::user_agent::UserAgent;

    fn sample_url() -> Url {
        "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
            .parse()
            .expect("A hardcoded valid URL string should always parse")
    }

    fn sample_user_agent() -> UserAgent {
        UserAgent::new("arkad contact@example.com")
            .expect("A hardcoded SEC-format string should always validate")
    }

    fn sample_raw_document() -> RawDocument<RawCompanyFacts> {
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");
        let metadata = DocumentMetadata::builder()
            .document_type::<RawDocument<RawCompanyFacts>>()
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
                url: sample_url(),
                http_status: StatusCode::Ok,
                user_agent: sample_user_agent(),
            })
            .build();
        RawDocument::from_parts(key, b"{}".to_vec(), metadata)
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_raw_document_of_raw_company_facts() {
        implements_auto_traits::<RawDocument<RawCompanyFacts>>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_raw_document_of_raw_company_facts() {
        implements_value_traits::<RawDocument<RawCompanyFacts>>();
    }

    const fn implements_document<T: Document>() {}
    #[test]
    const fn should_implement_document_for_raw_document_of_raw_company_facts() {
        implements_document::<RawDocument<RawCompanyFacts>>();
    }

    const fn implements_read_write_repository<T: ReadWriteRepository>() {}
    #[test]
    const fn should_implement_read_write_repository_for_a_filesystem_repository_of_raw_document() {
        implements_read_write_repository::<
            FilesystemRepository<ReadWrite, RawDocument<RawCompanyFacts>>,
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
            url: sample_url(),
            http_status: StatusCode::Ok,
            user_agent: sample_user_agent(),
        };

        let result = document.metadata().context().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_metadata_in_the_format_of_the_findings_for_raw_company_facts() {
        let produced_at = "2026-09-22T20:05:30Z"
            .parse::<DateTime<Utc>>()
            .expect("Given a hardcoded RFC 3339 timestamp, parsing it should always succeed");
        let metadata = DocumentMetadata::builder()
            .document_type::<RawDocument<RawCompanyFacts>>()
            .media_type("application/json")
            .content(
                3_789_099,
                "73a86c6aedc31f77cac2ea4df5f80f0b73a86c6aedc31f77cac2ea4df5f80f0b",
            )
            .produced_at(produced_at)
            .run(
                Uuid::from_u128(0x0b6f_3c1e_5d2a_4f7b_9c3e_8a1d_2b4c_6e8f),
                Producer::new("Store Company Facts", "0.1.0", "63c1f59"),
            )
            .context(RawDocumentMetadataContext {
                url: sample_url(),
                http_status: StatusCode::Ok,
                user_agent: sample_user_agent(),
            })
            .build();

        let expected_result = serde_json::json!({
            "metadata_version": 1,
            "kind": "raw_company_facts",
            "kind_version": 1,
            "media_type": "application/json",
            "bytes": 3_789_099,
            "sha256": "73a86c6aedc31f77cac2ea4df5f80f0b73a86c6aedc31f77cac2ea4df5f80f0b",
            "produced_at": "2026-09-22T20:05:30Z",
            "execution_id": "0b6f3c1e-5d2a-4f7b-9c3e-8a1d2b4c6e8f",
            "producer": {
                "name": "Store Company Facts",
                "crate_version": "0.1.0",
                "commit": "63c1f59",
            },
            "inputs": [],
            "context": {
                "url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json",
                "http_status": 200,
                "user_agent": "arkad contact@example.com",
            },
        });

        let result = serde_json::to_value(metadata)
            .expect("Given metadata of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }
}
