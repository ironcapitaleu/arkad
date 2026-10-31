//! # Store Notes Archive State
//!
//! Provides the [`StoreNotesArchive`] state, which writes one downloaded Notes file to the store
//! and outputs a [`NotesArchiveRef`] to the stored file. [`notes_key`] gives the key of the file
//! of a period.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`], and the [`NOTES_KEY_PREFIX`].
//! - [`context`]: The [`StoreNotesArchiveContext`] carried alongside the state.
//! - [`data`]: The [`StoreNotesArchiveInput`] and [`StoreNotesArchiveOutput`] data types.
//!
//! ## See Also
//!
//! - [`crate::shared::raw_document`]: The [`RawDocument`] type the state writes.

use std::fmt;

use async_trait::async_trait;
use chrono::Utc;
use serde::Serialize;
use state_maschine::prelude::State as SMState;
use storage::implementations::repository::filesystem::{Document, DocumentKey, IngestionMode};

use crate::error::State as StateError;
use crate::error::state_machine::state::FailedDocumentWrite;
use crate::shared::build_info::producer;
use crate::shared::document_storage::FailedDocumentStorage;
use crate::shared::notes_archive_ref::NotesArchiveRef;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawNotesArchive;
use crate::shared::run_fields::RunFields;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::{NOTES_KEY_PREFIX, STATE_NAME};
pub use context::StoreNotesArchiveContext;
pub use data::StoreNotesArchiveInput;
pub use data::StoreNotesArchiveOutput;

/// Writes one downloaded Notes file to the store.
///
/// Builds one [`RawDocument<RawNotesArchive>`](crate::shared::raw_source::RawNotesArchive) with
/// [`RawDocument::from_file`], writes it under the key `sec/notes/<file name>` through the
/// repository of type `R`, and outputs a [`NotesArchiveRef`]. The metadata records the ingestion
/// mode [`IngestionMode::Backfill`].
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct StoreNotesArchive<R> {
    input: StoreNotesArchiveInput,
    context: StoreNotesArchiveContext<R>,
    output: Option<StoreNotesArchiveOutput>,
}

impl<R> StoreNotesArchive<R> {
    /// Creates a new [`StoreNotesArchive`] state with no output computed yet.
    #[must_use]
    pub const fn new(input: StoreNotesArchiveInput, context: StoreNotesArchiveContext<R>) -> Self {
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
        StoreNotesArchiveInput,
        Option<StoreNotesArchiveOutput>,
        StoreNotesArchiveContext<R>,
    ) {
        (self.input, self.output, self.context)
    }
}

/// Returns the key `sec/notes/<file name>` of the Notes file of a period.
///
/// # Panics
///
/// Never panics: an SEC file name holds no path separator, so the key path is always valid.
///
/// # Examples
///
/// ```
/// use sec::implementations::states::extract::store_notes_archive::notes_key;
/// use sec::shared::notes_period::NotesPeriod;
///
/// let period = NotesPeriod::new("2010q1").expect("A hardcoded published quarter should always parse");
///
/// let expected_result = "sec/notes/2010q1_notes_1.zip";
///
/// let result = notes_key(period).as_path().display().to_string();
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn notes_key(period: NotesPeriod) -> DocumentKey {
    DocumentKey::new(format!("{NOTES_KEY_PREFIX}{}", period.file_name()))
        .expect("Given an SEC file name without a path separator, the relative key path should always be valid")
}

#[async_trait]
impl<R: RawRepository<RawNotesArchive>> State for StoreNotesArchive<R> {
    /// Writes the Notes file to the repository and stores a reference to it as output.
    ///
    /// The write succeeds even if a clone of the response is alive. That clone keeps the body buffer
    /// alive, so a transition must never keep one. The input gives up its response even if the
    /// write fails.
    ///
    /// # Errors
    ///
    /// - Returns [`StateError::InvalidInput`] if the input holds no response.
    /// - Returns [`StateError::FailedDocumentWrite`] if the repository rejects the write.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let response = self.input.response.take().ok_or(StateError::InvalidInput)?;
        let period = self.input.notes_period;
        let key = notes_key(period);
        let document = RawDocument::from_file(
            key.clone(),
            &response,
            &self.input.requested_url,
            &self.input.user_agent,
            RunFields {
                produced_at: Utc::now(),
                execution_id: self.context.execution_id,
                producer: producer(STATE_NAME),
                ingestion_mode: IngestionMode::Backfill,
            },
        );
        let archive = NotesArchiveRef::new(
            period,
            key,
            document.metadata().size_in_bytes(),
            document.metadata().sha256().clone(),
        );

        self.context
            .repository
            .persist(document)
            .await
            .map_err(|e| FailedDocumentWrite::new(STATE_NAME, FailedDocumentStorage::from(e)))?;
        self.output = Some(StoreNotesArchiveOutput::new(archive));
        Ok(())
    }
}

impl<R: RawRepository<RawNotesArchive>> SMState for StoreNotesArchive<R> {
    type InputData = StoreNotesArchiveInput;
    type OutputData = StoreNotesArchiveOutput;
    type Context = StoreNotesArchiveContext<R>;

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

impl<R: RawRepository<RawNotesArchive>> fmt::Display for StoreNotesArchive<R> {
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
    use std::os::unix::fs::MetadataExt;
    use std::path::Path;
    use std::sync::Arc;
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use storage::implementations::repository::filesystem::{
        FilesystemRepository, METADATA_SUFFIX, ReadWrite,
    };
    use storage::{Sha256Digest, WriteError};
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::*;
    use crate::shared::content_type::ContentType;
    use crate::shared::document_storage::DocumentStorageErrorReason;
    use crate::shared::headers::Headers;
    use crate::shared::raw_source::RawNotesArchive;
    use crate::shared::response::SecResponse as SecResponseTrait;
    use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use crate::tests::fixtures::sample_write_repository::FakeNotesRepository;

    type NotesFilesystemRepository = FilesystemRepository<ReadWrite, RawDocument<RawNotesArchive>>;

    const CONFLICT_REASON: &str = "A document already exists under sec/notes/2010q1_notes_1.zip";

    fn notes_period() -> NotesPeriod {
        NotesPeriod::new("2010q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed")
    }

    fn sample_key() -> DocumentKey {
        DocumentKey::new("sec/notes/2010q1_notes_1.zip")
            .expect("Given a hardcoded valid relative path, the key should always build")
    }

    fn sample_response() -> Arc<SecResponse<ZipBody>> {
        let url: Url = "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2010q1_notes_1.zip"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed");
        let headers = Headers::new(HashMap::from([(
            "content-type".to_string(),
            "application/zip".to_string(),
        )]));
        let response = SecResponse::<ZipBody>::from_parts(
            url,
            headers,
            ContentType::Zip,
            StatusCode::Ok,
            SAMPLE_NOTES_ZIP.to_vec(),
        )
        .expect(
            "Given status 200 and a ZIP content type, building the response should always succeed",
        );
        Arc::new(response)
    }

    fn sample_input(response: Arc<SecResponse<ZipBody>>) -> StoreNotesArchiveInput {
        let requested_url = response.url().clone();
        let user_agent = UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        StoreNotesArchiveInput::new(response, notes_period(), requested_url, user_agent)
    }

    fn sample_state<R>(repository: R) -> StoreNotesArchive<R> {
        StoreNotesArchive::new(
            sample_input(sample_response()),
            StoreNotesArchiveContext::new(Uuid::nil(), repository),
        )
    }

    fn filesystem_repository(root: &Path) -> NotesFilesystemRepository {
        FilesystemRepository::builder()
            .read_write()
            .document_type::<RawDocument<RawNotesArchive>>()
            .root(root)
            .build()
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    /// Runs a new state against the repository and panics if the store fails.
    async fn store(repository: NotesFilesystemRepository) {
        sample_state(repository)
            .compute_output_data_async()
            .await
            .expect("Given a fresh writable temp directory, the store should always succeed");
    }

    /// Returns the inode numbers of the document file and the metadata file under the key.
    ///
    /// A write renames a new file into place, so a rewritten file gets a new inode.
    fn inodes(repository: &NotesFilesystemRepository) -> (u64, u64) {
        let document_path = repository.document_path(&sample_key());
        let mut metadata_path = document_path.clone().into_os_string();
        metadata_path.push(METADATA_SUFFIX);
        let inode = |path: &Path| {
            std::fs::metadata(path)
                .expect("Given a file this test just stored, reading its metadata should always succeed")
                .ino()
        };
        (inode(&document_path), inode(Path::new(&metadata_path)))
    }

    #[test]
    fn should_return_state_name_for_store_notes_archive() {
        let state = sample_state(FakeNotesRepository::new());

        let expected_result = "Store Notes Archive";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let state = sample_state(FakeNotesRepository::new());

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_archive_reference_when_the_store_succeeds() {
        let root = temporary_root();
        let mut state = sample_state(filesystem_repository(root.path()));

        let expected_result = Some(&StoreNotesArchiveOutput::new(NotesArchiveRef::new(
            notes_period(),
            sample_key(),
            SAMPLE_NOTES_ZIP.len() as u64,
            Sha256Digest::of(SAMPLE_NOTES_ZIP),
        )));

        state
            .compute_output_data_async()
            .await
            .expect("Given a fresh writable temp directory, the store should always succeed");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_zip_bytes_unchanged_under_the_notes_key() {
        let root = temporary_root();
        let repository = filesystem_repository(root.path());
        store(repository.clone()).await;

        let expected_result = SAMPLE_NOTES_ZIP;

        let result = std::fs::read(repository.document_path(&sample_key())).expect(
            "Given a document this test just stored under the key, reading it should always succeed",
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_no_file_when_storing_equal_bytes_a_second_time() {
        let root = temporary_root();
        let repository = filesystem_repository(root.path());
        store(repository.clone()).await;

        let expected_result = inodes(&repository);

        store(repository.clone()).await;
        let result = inodes(&repository);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_backfill_as_the_ingestion_mode_for_notes_archives() {
        let repository = FakeNotesRepository::new();
        let mut state = sample_state(repository.clone());
        state
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
    async fn should_leave_the_input_without_a_response_when_the_store_succeeds() {
        let mut state = sample_state(FakeNotesRepository::new());
        state
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");

        let expected_result = true;

        let result = state.input_data().response.is_none();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_archive_when_a_clone_of_the_response_is_alive() {
        let response = sample_response();
        let _clone = Arc::clone(&response);
        let mut state = StoreNotesArchive::new(
            sample_input(response),
            StoreNotesArchiveContext::new(Uuid::nil(), FakeNotesRepository::new()),
        );

        let expected_result = Ok(());

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_input_when_the_input_holds_no_response() {
        let mut state = sample_state(FakeNotesRepository::new());
        state
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");

        let expected_result = Err(StateError::InvalidInput);

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_document_write_when_the_repository_rejects_the_write() {
        let write_error = WriteError::conflicting_write(CONFLICT_REASON);
        let mut state = sample_state(FakeNotesRepository::rejecting(write_error));

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
    async fn should_leave_the_input_without_a_response_when_the_repository_rejects_the_write() {
        let write_error = WriteError::conflicting_write(CONFLICT_REASON);
        let mut state = sample_state(FakeNotesRepository::rejecting(write_error));
        let _ = state.compute_output_data_async().await;

        let expected_result = true;

        let result = state.input_data().response.is_none();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_have_no_output_data_when_the_repository_rejects_the_write() {
        let write_error = WriteError::conflicting_write(CONFLICT_REASON);
        let mut state = sample_state(FakeNotesRepository::rejecting(write_error));

        let expected_result = false;

        let _ = state.compute_output_data_async().await;
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let mut state = sample_state(FakeNotesRepository::new());

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
    const fn should_implement_value_traits_for_store_notes_archive() {
        implements_value_traits::<StoreNotesArchive<FakeNotesRepository>>();
    }

    const fn implements_state<T: State>() {}
    #[test]
    const fn should_implement_state_for_store_notes_archive_of_a_filesystem_repository() {
        implements_state::<StoreNotesArchive<NotesFilesystemRepository>>();
    }
}
