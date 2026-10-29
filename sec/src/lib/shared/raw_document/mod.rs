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

use crate::shared::document_type::SecDocumentType;
use crate::shared::raw_source::RawSource;
use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::shared::run_fields::RunFields;
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;

pub mod raw_document_metadata_context;

pub use raw_document_metadata_context::RawDocumentMetadataContext;

/// One SEC file of the source `S`, the key naming it, and the metadata of the fetch that produced
/// it.
///
/// `S` sets the document type, so `RawDocument<RawCompanyFacts>` and a raw document of another file
/// are different types. A repository of one rejects the files of the other, as long as each
/// marker names its own document type.
///
/// # Examples
///
/// ```rust
/// use chrono::{DateTime, Utc};
/// use sec::shared::ingestion_mode::IngestionMode;
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
/// let bytes = b"{}";
/// let metadata = DocumentMetadata::builder()
///     .document_type::<RawDocument<RawCompanyFacts>>()
///     .media_type("application/json")
///     .content(bytes)
///     .produced_at(DateTime::<Utc>::UNIX_EPOCH)
///     .run(Uuid::nil(), Producer::new("Store Company Facts", "0.1.0", "63c1f59"))
///     .ingestion_mode(IngestionMode::Backfill)
///     .context(RawDocumentMetadataContext {
///         requested_url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
///             .parse()
///             .expect("A hardcoded valid URL string should always parse"),
///         final_url: "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
///             .parse()
///             .expect("A hardcoded valid URL string should always parse"),
///         http_status: StatusCode::Ok,
///         user_agent: UserAgent::new("arkad contact@example.com")
///             .expect("A hardcoded SEC-format string should always validate"),
///     })
///     .build();
///
/// let document = RawDocument::<RawCompanyFacts>::from_parts(key, bytes.to_vec(), metadata);
///
/// let expected_result = "raw_company_facts";
///
/// let result = document.metadata().document_type();
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

impl<S: RawSource> RawDocument<S> {
    /// Creates a new [`RawDocument`] from an SEC response and the fetch that produced it.
    ///
    /// The bytes, the size, and the SHA-256 digest all come from [`SecResponse::raw_body`], so the
    /// document holds the body exactly as the SEC sent it. The media type comes from the response.
    /// `run` names the run that fetched the file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// use chrono::{DateTime, Utc};
    /// use sec::shared::build_info::producer;
    /// use sec::shared::content_type::ContentType;
    /// use sec::shared::headers::Headers;
    /// use sec::shared::ingestion_mode::IngestionMode;
    /// use sec::shared::raw_document::RawDocument;
    /// use sec::shared::raw_source::RawCompanyFacts;
    /// use sec::shared::response::implementations::sec_response::SecResponse;
    /// use sec::shared::run_fields::RunFields;
    /// use sec::shared::status_code::StatusCode;
    /// use sec::shared::url::Url;
    /// use sec::shared::user_agent::UserAgent;
    /// use storage::implementations::repository::filesystem::{Document, DocumentKey};
    /// use uuid::Uuid;
    ///
    /// let url: Url = "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
    ///     .parse()
    ///     .expect("A hardcoded valid URL string should always parse");
    /// let response = SecResponse::from_parts(
    ///     url.clone(),
    ///     Headers::new(HashMap::new()),
    ///     ContentType::Json,
    ///     StatusCode::Ok,
    ///     b"{ \"price\": 1.50 }".to_vec(),
    /// )
    /// .expect("Given a valid JSON body, building the response should always succeed");
    /// let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
    ///     .expect("Given a valid relative path, the key should always build");
    /// let user_agent = UserAgent::new("arkad contact@example.com")
    ///     .expect("A hardcoded SEC-format string should always validate");
    ///
    /// let document = RawDocument::<RawCompanyFacts>::from_response(
    ///     key,
    ///     &response,
    ///     &url,
    ///     &user_agent,
    ///     RunFields {
    ///         produced_at: DateTime::<Utc>::UNIX_EPOCH,
    ///         execution_id: Uuid::nil(),
    ///         producer: producer("Store Company Facts"),
    ///         ingestion_mode: IngestionMode::Backfill,
    ///     },
    /// );
    ///
    /// let expected_result = b"{ \"price\": 1.50 }";
    ///
    /// let result = document.bytes();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn from_response(
        key: DocumentKey,
        response: &SecResponse,
        requested_url: &Url,
        user_agent: &UserAgent,
        run: RunFields,
    ) -> Self {
        let raw_body = response.raw_body();
        let metadata = DocumentMetadata::builder()
            .document_type::<Self>()
            .media_type(response.content_type().to_string())
            .content(raw_body)
            .produced_at(run.produced_at)
            .run(run.execution_id, run.producer)
            .ingestion_mode(run.ingestion_mode)
            .context(RawDocumentMetadataContext::new(
                response,
                requested_url,
                user_agent,
            ))
            .build();
        Self::from_parts(key, raw_body.to_vec(), metadata)
    }
}

impl<S: RawSource> Document for RawDocument<S> {
    type DocumentType = SecDocumentType;
    const DOCUMENT_TYPE: SecDocumentType = S::DOCUMENT_TYPE;
    const DOCUMENT_TYPE_VERSION: u32 = S::DOCUMENT_TYPE_VERSION;
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
    use std::collections::HashMap;
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
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::ingestion_mode::IngestionMode;
    use crate::shared::raw_source::RawCompanyFacts;
    use crate::shared::status_code::StatusCode;

    // Whitespace and a trailing zero that a parse and a reserialization drop.
    const SAMPLE_RAW_BODY: &[u8] = b"{ \"price\": 1.50 }";

    fn sample_redirected_url() -> Url {
        "https://data.sec.gov/api/xbrl/companyfacts/redirected/CIK0000320193.json"
            .parse()
            .expect("A hardcoded valid URL string should always parse")
    }

    fn sample_response() -> SecResponse {
        SecResponse::from_parts(
            sample_redirected_url(),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            SAMPLE_RAW_BODY.to_vec(),
        )
        .expect("Given a hardcoded valid JSON body, building the response should always succeed")
    }

    fn sample_document_from_response() -> RawDocument<RawCompanyFacts> {
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");
        RawDocument::from_response(
            key,
            &sample_response(),
            &sample_url(),
            &sample_user_agent(),
            RunFields {
                produced_at: DateTime::<Utc>::UNIX_EPOCH,
                execution_id: Uuid::nil(),
                producer: Producer::new("Store Company Facts", "0.1.0", "63c1f59"),
                ingestion_mode: IngestionMode::Backfill,
            },
        )
    }

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
            .content(b"{}")
            .produced_at(DateTime::<Utc>::UNIX_EPOCH)
            .run(
                Uuid::nil(),
                Producer::new("Store Company Facts", "0.1.0", "63c1f59"),
            )
            .ingestion_mode(IngestionMode::Backfill)
            .context(RawDocumentMetadataContext {
                requested_url: sample_url(),
                final_url: sample_url(),
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
    fn should_store_the_raw_company_facts_document_type_when_building_its_metadata() {
        let document = sample_raw_document();

        let expected_result = "raw_company_facts";

        let result = document.metadata().document_type();

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
            requested_url: sample_url(),
            final_url: sample_url(),
            http_status: StatusCode::Ok,
            user_agent: sample_user_agent(),
        };

        let result = document.metadata().context().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_metadata_in_the_stored_format_for_raw_company_facts() {
        let produced_at = "2026-09-22T20:05:30Z"
            .parse::<DateTime<Utc>>()
            .expect("Given a hardcoded RFC 3339 timestamp, parsing it should always succeed");
        let metadata = DocumentMetadata::builder()
            .document_type::<RawDocument<RawCompanyFacts>>()
            .media_type("application/json")
            .content(b"{}")
            .produced_at(produced_at)
            .run(
                Uuid::from_u128(0x0b6f_3c1e_5d2a_4f7b_9c3e_8a1d_2b4c_6e8f),
                Producer::new("Store Company Facts", "0.1.0", "63c1f59"),
            )
            .ingestion_mode(IngestionMode::Backfill)
            .context(RawDocumentMetadataContext {
                requested_url: sample_url(),
                final_url: sample_url(),
                http_status: StatusCode::Ok,
                user_agent: sample_user_agent(),
            })
            .build();

        let expected_result = serde_json::json!({
            "metadata_version": 1,
            "document_type": "raw_company_facts",
            "document_type_version": 1,
            "media_type": "application/json",
            "bytes": 2,
            "sha256": "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
            "produced_at": "2026-09-22T20:05:30Z",
            "execution_id": "0b6f3c1e-5d2a-4f7b-9c3e-8a1d2b4c6e8f",
            "ingestion_mode": "backfill",
            "producer": {
                "name": "Store Company Facts",
                "crate_version": "0.1.0",
                "commit": "63c1f59",
            },
            "inputs": [],
            "context": {
                "requested_url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json",
                "final_url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json",
                "http_status": 200,
                "user_agent": "arkad contact@example.com",
            },
        });

        let result = serde_json::to_value(metadata)
            .expect("Given metadata of domain types, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_the_raw_body_unchanged_when_building_a_document_from_a_response() {
        let document = sample_document_from_response();

        let expected_result = SAMPLE_RAW_BODY;

        let result = document.bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_size_from_the_raw_body_when_building_a_document_from_a_response() {
        let document = sample_document_from_response();

        let expected_result = SAMPLE_RAW_BODY.len() as u64;

        let result = document.metadata().size_in_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_digest_from_the_raw_body_when_building_a_document_from_a_response() {
        let document = sample_document_from_response();

        let expected_result = storage::Sha256Digest::of(SAMPLE_RAW_BODY);

        let result = document.metadata().sha256().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_media_type_from_the_response_when_building_a_document_from_a_response() {
        let document = sample_document_from_response();

        let expected_result = "application/json";

        let result = document.metadata().media_type();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_record_both_urls_and_the_user_agent_when_building_a_document_from_a_response() {
        let document = sample_document_from_response();

        let expected_result = RawDocumentMetadataContext {
            requested_url: sample_url(),
            final_url: sample_redirected_url(),
            http_status: StatusCode::Ok,
            user_agent: sample_user_agent(),
        };

        let result = document.metadata().context().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_record_the_producer_when_building_a_document_from_a_response() {
        let document = sample_document_from_response();

        let expected_result = Producer::new("Store Company Facts", "0.1.0", "63c1f59");

        let result = document.metadata().producer().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_record_the_execution_id_when_building_a_document_from_a_response() {
        let document = sample_document_from_response();

        let expected_result = Uuid::nil();

        let result = document.metadata().execution_id();

        assert_eq!(result, expected_result);
    }
}
