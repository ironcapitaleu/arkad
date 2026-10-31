//! # Store Daily Index State
//!
//! Provides the [`StoreDailyIndex`] state, which writes the downloaded daily index of one day to
//! the store and outputs its key and its text.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`], and the [`DAILY_INDEX_KEY_PREFIX`].
//! - [`context`]: The [`StoreDailyIndexContext`] carried alongside the state.
//! - [`data`]: The [`StoreDailyIndexInput`] and [`StoreDailyIndexOutput`] data types.
//!
//! ## See Also
//!
//! - [`crate::shared::raw_document`]: The [`RawDocument`] type the state writes.

use std::fmt;

use async_trait::async_trait;
use chrono::Utc;
use serde::Serialize;
use state_maschine::prelude::State as SMState;
use storage::implementations::repository::filesystem::{DocumentKey, IngestionMode};

use crate::error::State as StateError;
use crate::error::state_machine::state::FailedDocumentWrite;
use crate::shared::build_info::producer;
use crate::shared::document_storage::FailedDocumentStorage;
use crate::shared::index_date::IndexDate;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawDailyIndex;
use crate::shared::run_fields::RunFields;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::{DAILY_INDEX_KEY_PREFIX, STATE_NAME};
pub use context::StoreDailyIndexContext;
pub use data::{StoreDailyIndexInput, StoreDailyIndexOutput, StoredDailyIndex};

/// Writes the downloaded daily index of one day to the store.
///
/// Builds one [`RawDocument<RawDailyIndex>`] with [`RawDocument::from_file`], writes it under the
/// key `sec/daily-index/form.YYYYMMDD.idx` through the repository of type `R`, and outputs the key
/// and the text. The metadata records the ingestion mode [`IngestionMode::Incremental`]. For a day
/// without filings, the state writes nothing and outputs no index.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct StoreDailyIndex<R> {
    input: StoreDailyIndexInput,
    context: StoreDailyIndexContext<R>,
    output: Option<StoreDailyIndexOutput>,
}

impl<R> StoreDailyIndex<R> {
    /// Creates a new [`StoreDailyIndex`] state with no output computed yet.
    #[must_use]
    pub const fn new(input: StoreDailyIndexInput, context: StoreDailyIndexContext<R>) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        StoreDailyIndexInput,
        Option<StoreDailyIndexOutput>,
        StoreDailyIndexContext<R>,
    ) {
        (self.input, self.output, self.context)
    }
}

/// Returns the key `sec/daily-index/form.YYYYMMDD.idx` of the daily index of a date.
fn daily_index_key(date: IndexDate) -> DocumentKey {
    DocumentKey::new(format!(
        "{DAILY_INDEX_KEY_PREFIX}form.{}.idx",
        date.basic_format()
    ))
    .expect("Given a file name of digits without a path separator, the relative key path should always be valid")
}

#[async_trait]
impl<R: RawRepository<RawDailyIndex>> State for StoreDailyIndex<R> {
    /// Writes the daily index to the repository and stores its key and text as output.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::FailedDocumentWrite`] if the repository rejects the write.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let Some(response) = self.input.response.as_deref() else {
            self.output = Some(StoreDailyIndexOutput::without_filings());
            return Ok(());
        };
        let key = daily_index_key(self.input.index_date);
        let document = RawDocument::<RawDailyIndex>::from_file(
            key.clone(),
            response,
            &self.input.requested_url,
            &self.input.user_agent,
            RunFields {
                produced_at: Utc::now(),
                execution_id: self.context.execution_id,
                producer: producer(STATE_NAME),
                ingestion_mode: IngestionMode::Incremental,
            },
        );
        let index = StoredDailyIndex {
            key,
            text: response.body_bytes(),
        };

        self.context
            .repository
            .persist(document)
            .await
            .map_err(|e| FailedDocumentWrite::new(STATE_NAME, FailedDocumentStorage::from(e)))?;
        self.output = Some(StoreDailyIndexOutput::new(index));
        Ok(())
    }
}

impl<R: RawRepository<RawDailyIndex>> SMState for StoreDailyIndex<R> {
    type InputData = StoreDailyIndexInput;
    type OutputData = StoreDailyIndexOutput;
    type Context = StoreDailyIndexContext<R>;

    fn state_name(&self) -> impl ToString {
        STATE_NAME
    }

    /// Blocking wrapper around [`compute_output_data_async`](crate::traits::state_machine::state::State::compute_output_data_async).
    ///
    /// # Panics
    ///
    /// - Panics if the async computation returns an error.
    /// - Panics if called inside a `current_thread` runtime, because `block_in_place` requires a
    ///   multi-thread runtime.
    fn compute_output_data(&mut self) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(self.compute_output_data_async()))
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.compute_output_data_async())
        };

        if let Err(e) = result {
            panic!("compute_output_data failed: {e}")
        }
    }

    fn context_data(&self) -> &Self::Context {
        &self.context
    }

    fn input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl<R: RawRepository<RawDailyIndex>> fmt::Display for StoreDailyIndex<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Arc;
    use std::{fmt::Debug, hash::Hash};

    use bytes::Bytes;
    use pretty_assertions::assert_eq;
    use storage::WriteError;
    use storage::implementations::repository::filesystem::{
        Document, FilesystemRepository, ReadWrite,
    };
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::*;
    use crate::shared::content_type::ContentType;
    use crate::shared::document_storage::DocumentStorageErrorReason;
    use crate::shared::headers::Headers;
    use crate::shared::response::SecResponse as SecResponseTrait;
    use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_write_repository::FakeDailyIndexRepository;

    type DailyIndexFilesystemRepository =
        FilesystemRepository<ReadWrite, RawDocument<RawDailyIndex>>;

    const SAMPLE_INDEX: &[u8] = b"Form Type   Company Name        CIK         Date Filed  File Name\n\
        ---------------------------------------------------------------------------------\n\
        10-Q        APPLE INC           320193      20261027    edgar/data/320193/0000320193-26-000101.txt\n";

    const CONFLICT_REASON: &str =
        "A document already exists under sec/daily-index/form.20261027.idx";

    fn index_date() -> IndexDate {
        IndexDate::new("2026-10-27")
            .expect("Given a hardcoded past date, parsing should always succeed")
    }

    fn sample_key() -> DocumentKey {
        DocumentKey::new("sec/daily-index/form.20261027.idx")
            .expect("Given a hardcoded valid relative path, the key should always build")
    }

    fn sample_response() -> Arc<SecResponse<ZipBody>> {
        let url: Url = "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261027.idx"
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
            SAMPLE_INDEX.to_vec(),
        )
        .expect(
            "Given status 200 and an octet stream, building the response should always succeed",
        );
        Arc::new(response)
    }

    fn sample_input(response: Option<Arc<SecResponse<ZipBody>>>) -> StoreDailyIndexInput {
        let requested_url = sample_response().url().clone();
        let user_agent = UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        StoreDailyIndexInput::new(response, index_date(), requested_url, user_agent)
    }

    fn sample_state<R>(repository: R) -> StoreDailyIndex<R> {
        StoreDailyIndex::new(
            sample_input(Some(sample_response())),
            StoreDailyIndexContext::new(Uuid::nil(), repository),
        )
    }

    fn state_without_filings<R>(repository: R) -> StoreDailyIndex<R> {
        StoreDailyIndex::new(
            sample_input(None),
            StoreDailyIndexContext::new(Uuid::nil(), repository),
        )
    }

    fn filesystem_repository(root: &Path) -> DailyIndexFilesystemRepository {
        FilesystemRepository::builder()
            .read_write()
            .document_type::<RawDocument<RawDailyIndex>>()
            .root(root)
            .build()
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    /// Runs the state against a fake repository and returns the one document it persisted.
    async fn persisted_document() -> Option<RawDocument<RawDailyIndex>> {
        let repository = FakeDailyIndexRepository::new();
        sample_state(repository.clone())
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");
        repository.persisted().pop()
    }

    #[test]
    fn should_return_state_name_for_store_daily_index() {
        let state = sample_state(FakeDailyIndexRepository::new());

        let expected_result = "Store Daily Index";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let state = sample_state(FakeDailyIndexRepository::new());

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_the_key_from_the_basic_format_of_the_index_date() {
        let date = index_date();

        let expected_result = sample_key();

        let result = daily_index_key(date);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_key_and_the_text_when_the_store_succeeds() {
        let root = temporary_root();
        let mut state = sample_state(filesystem_repository(root.path()));

        let expected_result = Some(&StoreDailyIndexOutput::new(StoredDailyIndex {
            key: sample_key(),
            text: Bytes::from_static(SAMPLE_INDEX),
        }));

        state
            .compute_output_data_async()
            .await
            .expect("Given a fresh writable temp directory, the store should always succeed");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_index_bytes_unchanged_under_the_daily_index_key() {
        let root = temporary_root();
        let repository = filesystem_repository(root.path());
        sample_state(repository.clone())
            .compute_output_data_async()
            .await
            .expect("Given a fresh writable temp directory, the store should always succeed");

        let expected_result = SAMPLE_INDEX;

        let result = std::fs::read(repository.document_path(&sample_key())).expect(
            "Given a document this test just stored under the key, reading it should always succeed",
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_incremental_as_the_ingestion_mode_for_daily_indexes() {
        let expected_result = Some(IngestionMode::Incremental);

        let result = persisted_document()
            .await
            .map(|document| document.metadata().ingestion_mode());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_raw_daily_index_as_the_document_type_for_daily_indexes() {
        let expected_result = Some("raw_daily_index".to_string());

        let result = persisted_document()
            .await
            .map(|document| document.metadata().document_type().to_string());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_no_index_for_a_day_without_filings() {
        let mut state = state_without_filings(FakeDailyIndexRepository::new());

        let expected_result = Some(&StoreDailyIndexOutput::without_filings());

        state
            .compute_output_data_async()
            .await
            .expect("Given a day without filings, the state should succeed without a write");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_no_document_for_a_day_without_filings() {
        let repository = FakeDailyIndexRepository::new();
        state_without_filings(repository.clone())
            .compute_output_data_async()
            .await
            .expect("Given a day without filings, the state should succeed without a write");

        let expected_result = 0;

        let result = repository.persisted().len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_succeed_for_a_day_without_filings_when_the_repository_rejects_writes() {
        let write_error = WriteError::conflicting_write(CONFLICT_REASON);
        let mut state = state_without_filings(FakeDailyIndexRepository::rejecting(write_error));

        let expected_result = Ok(());

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_document_write_when_the_repository_rejects_the_write() {
        let write_error = WriteError::conflicting_write(CONFLICT_REASON);
        let mut state = sample_state(FakeDailyIndexRepository::rejecting(write_error));

        let expected_result = Err(StateError::FailedDocumentWrite(FailedDocumentWrite::new(
            STATE_NAME,
            FailedDocumentStorage::new(DocumentStorageErrorReason::ConflictingWrite {
                details: CONFLICT_REASON.to_string(),
            }),
        )));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_have_no_output_data_when_the_repository_rejects_the_write() {
        let write_error = WriteError::conflicting_write(CONFLICT_REASON);
        let mut state = sample_state(FakeDailyIndexRepository::rejecting(write_error));

        let expected_result = false;

        let _ = state.compute_output_data_async().await;
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_index_again_when_the_state_runs_a_second_time() {
        let repository = FakeDailyIndexRepository::new();
        let mut state = sample_state(repository.clone());
        state
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");
        state
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");

        let expected_result = 2;

        let result = repository.persisted().len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let mut state = sample_state(FakeDailyIndexRepository::new());

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_daily_index() {
        implements_value_traits::<StoreDailyIndex<FakeDailyIndexRepository>>();
    }

    const fn implements_state<T: State>() {}
    #[test]
    const fn should_implement_state_for_store_daily_index_of_a_filesystem_repository() {
        implements_state::<StoreDailyIndex<DailyIndexFilesystemRepository>>();
    }
}
