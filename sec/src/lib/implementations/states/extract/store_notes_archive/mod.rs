//! # Store Notes Archive State
//!
//! Provides [`StoreNotesArchive`], the [`StoreSecResponse`] state of the [`NotesFile`] kind, which
//! writes one downloaded Notes file to the store and outputs a [`NotesArchiveRef`] to the stored
//! file. [`notes_key`] gives the key of the file of a period.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`], and the [`NOTES_KEY_PREFIX`].
//! - [`data`]: The [`StoreNotesArchiveOutput`] data type.
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract::store_sec_response`]: The generic store state.
//! - [`crate::shared::raw_document`]: The [`RawDocument`] type the state writes.

use storage::implementations::repository::filesystem::{Document, DocumentKey};

use crate::implementations::states::extract::store_sec_response::{
    StoreSecResponse, StoreSecResponseContext, StoreSecResponseInput,
};
use crate::shared::notes_archive_ref::NotesArchiveRef;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::raw_document::RawDocument;
use crate::shared::raw_source::RawNotesArchive;
use crate::shared::request::implementations::sec_request::{NotesFile, StoredAs};
use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
use crate::shared::run_fields::RunFields;
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;

pub mod constants;
pub mod data;

pub use constants::{NOTES_KEY_PREFIX, STATE_NAME};
pub use data::StoreNotesArchiveOutput;

/// The [`StoreSecResponse`] state of the [`NotesFile`] kind.
///
/// It writes one [`RawDocument<RawNotesArchive>`] under the key `sec/notes/<file name>` and
/// outputs a [`NotesArchiveRef`] to it.
pub type StoreNotesArchive<R> = StoreSecResponse<NotesFile, R>;

/// The input of [`StoreNotesArchive`].
pub type StoreNotesArchiveInput = StoreSecResponseInput<NotesFile>;

/// The context of [`StoreNotesArchive`].
pub type StoreNotesArchiveContext<R> = StoreSecResponseContext<NotesFile, R>;

/// Builds the document with [`RawDocument::from_file`], which keeps the ZIP bytes unchanged.
impl StoredAs for NotesFile {
    type Source = RawNotesArchive;
    type Output = StoreNotesArchiveOutput;

    const STATE_NAME: &'static str = STATE_NAME;

    fn store(
        response: &SecResponse<ZipBody>,
        period: &NotesPeriod,
        requested_url: &Url,
        user_agent: &UserAgent,
        run: RunFields,
    ) -> (RawDocument<RawNotesArchive>, StoreNotesArchiveOutput) {
        let key = notes_key(*period);
        let document =
            RawDocument::from_file(key.clone(), response, requested_url, user_agent, run);
        let archive = NotesArchiveRef::new(
            *period,
            key,
            document.metadata().size_in_bytes(),
            document.metadata().sha256().clone(),
        );
        (document, StoreNotesArchiveOutput::new(archive))
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::os::unix::fs::MetadataExt;
    use std::path::Path;
    use std::sync::Arc;
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use state_maschine::prelude::State as SMState;
    use storage::implementations::repository::filesystem::{
        FilesystemRepository, IngestionMode, METADATA_SUFFIX, ReadWrite,
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
    use crate::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use crate::tests::fixtures::sample_write_repository::FakeNotesRepository;
    use crate::traits::state_machine::state::State;

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
        StoreNotesArchiveInput::new(response, requested_url, user_agent)
    }

    fn sample_context<R>(
        ingestion_mode: IngestionMode,
        repository: R,
    ) -> StoreNotesArchiveContext<R> {
        StoreNotesArchiveContext::new(notes_period(), Uuid::nil(), ingestion_mode, repository)
    }

    fn sample_state<R>(repository: R) -> StoreNotesArchive<R> {
        StoreNotesArchive::new(
            sample_input(sample_response()),
            sample_context(IngestionMode::Backfill, repository),
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
    async fn should_record_the_ingestion_mode_of_the_context_for_notes_archives() {
        let repository = FakeNotesRepository::new();
        let mut state = StoreNotesArchive::new(
            sample_input(sample_response()),
            sample_context(IngestionMode::Incremental, repository.clone()),
        );
        state
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");

        let expected_result = Some(IngestionMode::Incremental);

        let result = repository
            .persisted()
            .pop()
            .map(|document| document.metadata().ingestion_mode());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_archive_when_a_clone_of_the_response_is_alive() {
        let response = sample_response();
        let _clone = Arc::clone(&response);
        let mut state = StoreNotesArchive::new(
            sample_input(response),
            sample_context(IngestionMode::Backfill, FakeNotesRepository::new()),
        );

        let expected_result = Ok(());

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
