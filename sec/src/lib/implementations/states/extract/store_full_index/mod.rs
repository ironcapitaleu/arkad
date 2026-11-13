//! # Store Full Index State
//!
//! Provides [`StoreFullIndex`], the [`StoreSecResponse`] state of the [`FullIndex`] kind, which
//! writes the full index `form.idx` of one quarter to the store and outputs the stored index with
//! its text. [`full_index_key`] gives the key of the file of a quarter.
//!
//! The SEC adds rows to the file of the current quarter each day, and a later run replaces the
//! stored file under the same key.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`], and the parts of the document key.
//! - [`data`]: The [`StoreFullIndexOutput`] data type.
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract::store_sec_response`]: The generic store state.
//! - [`crate::shared::raw_document`]: The [`RawDocument`] type the state writes.

use storage::implementations::repository::filesystem::{Document, DocumentKey};

use crate::implementations::states::extract::store_sec_response::{
    StoreSecResponse, StoreSecResponseContext, StoreSecResponseInput,
};
use crate::shared::quarter::Quarter;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::RawFullIndex;
use crate::shared::request::implementations::sec_request::{FullIndex, StoredAs};
use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
use crate::shared::run_fields::RunFields;
use crate::shared::stored_form_index::StoredFormIndex;
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;

pub mod constants;
pub mod data;

pub use constants::{FULL_INDEX_KEY_FILE_NAME, FULL_INDEX_KEY_PREFIX, STATE_NAME};
pub use data::StoreFullIndexOutput;

/// The [`StoreSecResponse`] state of the [`FullIndex`] kind.
///
/// It writes one [`RawDocument<RawFullIndex>`] under the key `sec/full-index/YYYY/QTRn/form.idx`
/// and outputs the stored index with its text.
pub type StoreFullIndex<R> = StoreSecResponse<FullIndex, R>;

/// The input of [`StoreFullIndex`].
pub type StoreFullIndexInput = StoreSecResponseInput<FullIndex>;

/// The context of [`StoreFullIndex`].
pub type StoreFullIndexContext<R> = StoreSecResponseContext<FullIndex, R>;

/// Builds the document with [`RawDocument::from_file`], which keeps the bytes unchanged.
impl StoredAs for FullIndex {
    type Source = RawFullIndex;
    type Output = StoreFullIndexOutput;

    const STATE_NAME: &'static str = STATE_NAME;

    fn store(
        response: &SecResponse<ZipBody>,
        quarter: &Quarter,
        requested_url: &Url,
        user_agent: &UserAgent,
        run: RunFields,
    ) -> (RawDocument<RawFullIndex>, StoreFullIndexOutput) {
        let key = full_index_key(*quarter);
        let document =
            RawDocument::from_file(key.clone(), response, requested_url, user_agent, run);
        let index = StoredFormIndex {
            key,
            sha256: document.metadata().sha256().clone(),
            text: response.body_bytes(),
        };
        (document, StoreFullIndexOutput::new(index))
    }
}

/// Returns the key `sec/full-index/YYYY/QTRn/form.idx` of the full index of a quarter.
///
/// # Panics
///
/// Never panics: the year and the quarter folder hold no path separator, so the key path is always
/// valid.
///
/// # Examples
///
/// ```
/// use chrono::NaiveDate;
/// use sec::implementations::states::extract::store_full_index::full_index_key;
/// use sec::shared::quarter::Quarter;
///
/// let date = NaiveDate::from_ymd_opt(2026, 10, 1)
///     .expect("Given a hardcoded calendar date, the date should always build");
///
/// let expected_result = "sec/full-index/2026/QTR4/form.idx";
///
/// let result = full_index_key(Quarter::of(date)).as_path().display().to_string();
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn full_index_key(quarter: Quarter) -> DocumentKey {
    DocumentKey::new(format!(
        "{FULL_INDEX_KEY_PREFIX}{quarter}/{FULL_INDEX_KEY_FILE_NAME}"
    ))
    .expect("Given a year and a quarter folder without a path separator, the relative key path should always be valid")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::{fmt::Debug, hash::Hash};

    use bytes::Bytes;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::State as SMState;
    use storage::implementations::repository::filesystem::{
        FilesystemRepository, IngestionMode, InputDocument, ReadWrite,
    };
    use storage::{Sha256Digest, WriteError};
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::*;
    use crate::error::State as StateError;
    use crate::error::state_machine::state::FailedDocumentWrite;
    use crate::shared::content_type::ContentType;
    use crate::shared::document_storage::{DocumentStorageErrorReason, FailedDocumentStorage};
    use crate::shared::headers::Headers;
    use crate::shared::response::SecResponse as SecResponseTrait;
    use crate::shared::status_code::StatusCode;
    use crate::tests::fixtures::data::full_index::SAMPLE_FULL_INDEX_2026_Q4;
    use crate::tests::fixtures::sample_write_repository::FakeFullIndexRepository;
    use crate::traits::state_machine::state::State;

    type FullIndexFilesystemRepository = FilesystemRepository<ReadWrite, RawDocument<RawFullIndex>>;

    const KEY: &str = "sec/full-index/2026/QTR4/form.idx";

    const CONFLICT_REASON: &str =
        "A document already exists under sec/full-index/2026/QTR4/form.idx";

    fn quarter() -> Quarter {
        Quarter::of(
            NaiveDate::from_ymd_opt(2026, 11, 13)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    fn sample_key() -> DocumentKey {
        DocumentKey::new(KEY)
            .expect("Given a hardcoded valid relative path, the key should always build")
    }

    fn sample_response() -> Arc<SecResponse<ZipBody>> {
        let url: Url = "https://www.sec.gov/Archives/edgar/full-index/2026/QTR4/form.idx"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed");
        let headers = Headers::new(HashMap::from([(
            "content-type".to_string(),
            "application/octet-stream".to_string(),
        )]));
        let response = SecResponse::<ZipBody>::from_parts(
            url,
            headers,
            ContentType::OctetStream,
            StatusCode::Ok,
            SAMPLE_FULL_INDEX_2026_Q4.as_bytes().to_vec(),
        )
        .expect("Given status 200 and an octet-stream content type, building the response should always succeed");
        Arc::new(response)
    }

    fn sample_state<R>(ingestion_mode: IngestionMode, repository: R) -> StoreFullIndex<R> {
        let response = sample_response();
        let requested_url = response.url().clone();
        let user_agent = UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        StoreFullIndex::new(
            StoreFullIndexInput::new(response, requested_url, user_agent),
            StoreFullIndexContext::new(quarter(), Uuid::nil(), ingestion_mode, repository),
        )
    }

    fn filesystem_repository(root: &TempDir) -> FullIndexFilesystemRepository {
        FilesystemRepository::builder()
            .read_write()
            .document_type::<RawDocument<RawFullIndex>>()
            .root(root.path())
            .build()
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    #[test]
    fn should_return_state_name_for_store_full_index() {
        let state = sample_state(IngestionMode::Incremental, FakeFullIndexRepository::new());

        let expected_result = "Store Full Index";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_key_of_the_first_quarter_when_the_quarter_starts_a_year() {
        let quarter = Quarter::of(
            NaiveDate::from_ymd_opt(2027, 1, 1)
                .expect("Given a hardcoded calendar date, the date should always build"),
        );

        let expected_result = "sec/full-index/2027/QTR1/form.idx";

        let result = full_index_key(quarter).as_path().display().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_key_the_sha256_and_the_text_when_the_store_succeeds() {
        let root = temporary_root();
        let mut state = sample_state(IngestionMode::Incremental, filesystem_repository(&root));

        let expected_result = Some(&StoreFullIndexOutput::new(StoredFormIndex {
            key: sample_key(),
            sha256: Sha256Digest::of(SAMPLE_FULL_INDEX_2026_Q4.as_bytes()),
            text: Bytes::from_static(SAMPLE_FULL_INDEX_2026_Q4.as_bytes()),
        }));

        state
            .compute_output_data_async()
            .await
            .expect("Given a fresh writable temp directory, the store should always succeed");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_bytes_unchanged_under_the_full_index_key() {
        let root = temporary_root();
        let repository = filesystem_repository(&root);
        sample_state(IngestionMode::Incremental, repository.clone())
            .compute_output_data_async()
            .await
            .expect("Given a fresh writable temp directory, the store should always succeed");

        let expected_result = SAMPLE_FULL_INDEX_2026_Q4.as_bytes();

        let result = std::fs::read(repository.document_path(&sample_key())).expect(
            "Given a document this test just stored under the key, reading it should always succeed",
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_the_ingestion_mode_of_the_context_for_full_indexes() {
        let repository = FakeFullIndexRepository::new();
        sample_state(IngestionMode::Backfill, repository.clone())
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");

        let expected_result = Some(IngestionMode::Backfill);

        let result = repository
            .persisted()
            .pop()
            .map(|document| document.metadata().ingestion_mode());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_no_inputs_for_full_indexes() {
        let repository = FakeFullIndexRepository::new();
        sample_state(IngestionMode::Incremental, repository.clone())
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");

        let expected_result = Some(Vec::<InputDocument>::new());

        let result = repository
            .persisted()
            .pop()
            .map(|document| document.metadata().inputs().to_vec());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_document_write_when_the_repository_rejects_the_write() {
        let write_error = WriteError::conflicting_write(CONFLICT_REASON);
        let mut state = sample_state(
            IngestionMode::Incremental,
            FakeFullIndexRepository::rejecting(write_error),
        );

        let expected_result = Err(StateError::FailedDocumentWrite(FailedDocumentWrite::new(
            STATE_NAME,
            FailedDocumentStorage::new(DocumentStorageErrorReason::ConflictingWrite {
                details: CONFLICT_REASON.to_string(),
            }),
        )));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_full_index() {
        implements_value_traits::<StoreFullIndex<FakeFullIndexRepository>>();
    }

    const fn implements_state<T: State>() {}
    #[test]
    const fn should_implement_state_for_store_full_index_of_a_filesystem_repository() {
        implements_state::<StoreFullIndex<FullIndexFilesystemRepository>>();
    }
}
