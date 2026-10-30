//! # Extract Notes Super-State
//!
//! Provides the [`ExtractNotesSuperState`], which downloads one SEC Financial Statement and Notes
//! file and writes it to the store.
//!
//! The super-state owns the shared SEC client and the `execution_id`. Each state holds its own
//! clone of the storage repository.
//!
//! ## Modules
//!
//! - [`builder`]: Builds the [`ExtractNotesSuperState`] at its entry state
//!   [`ValidateNotesPeriod`].
//!
//! ## State Flow
//!
//! [`ValidateNotesPeriod`] → [`PrepareNotesRequest`] → [`DownloadNotesArchive`] →
//! [`StoreNotesArchive`]. The stream ends after [`StoreNotesArchive`].
//! [`ExtractNotesSuperState::run_to_archive`] runs the stream and returns the [`NotesArchiveRef`]
//! of the stored file.
//!
//! ## Usage
//!
//! ```no_run
//! use futures_util::StreamExt;
//! use sec::implementations::states::extract::notes::ExtractNotesSuperState;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use sec::shared::raw_document::RawDocument;
//! use sec::shared::raw_source::RawNotesArchive;
//! use storage::implementations::repository::filesystem::FilesystemRepository;
//! use uuid::Uuid;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let repository = FilesystemRepository::builder()
//!     .writer()
//!     .document_type::<RawDocument<RawNotesArchive>>()
//!     .root("data")
//!     .build();
//! let extract_state = ExtractNotesSuperState::builder()
//!     .period("2024q1")
//!     .sec_client(SecClient::default())
//!     .execution_id(Uuid::new_v4())
//!     .repository(repository)
//!     .build();
//! let mut stream = extract_state.into_run_stream();
//! while let Some(item) = stream.next().await {
//!     let _item = item.expect("The live SEC download should succeed");
//! }
//! # }
//! ```

pub mod builder;

use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use futures_util::StreamExt;
use serde::Serialize;
use state_maschine::prelude::{StateMachine as SMStateMachine, Transition as SMTransition};
use uuid::Uuid;

use crate::error::State as StateError;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::download_notes_archive::DownloadNotesArchive;
use crate::implementations::states::extract::notes::builder::{
    ExtractNotesSuperStateBuilder, NoExecutionId, NoPeriod, NoRepository, NoSecClient,
};
use crate::implementations::states::extract::notes_repository::NotesRepository;
use crate::implementations::states::extract::prepare_sec_request::PrepareNotesRequest;
use crate::implementations::states::extract::store_notes_archive::StoreNotesArchive;
use crate::implementations::states::extract::validate_notes_period::ValidateNotesPeriod;
use crate::prelude::*;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::notes_archive_ref::NotesArchiveRef;
use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
use crate::traits::state_machine::stream::terminal_stream_with;

/// Input and output data for the [`ExtractNotesSuperState`].
///
/// A placeholder with unit update semantics: the super-state delegates all real data handling to
/// its current inner state, so it carries no data of its own.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ExtractNotesSuperStateData;

impl SMStateData for ExtractNotesSuperStateData {
    type UpdateType = ();
    fn state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, (): Self::UpdateType) {}
}

impl StateData for ExtractNotesSuperStateData {
    fn update_state(&mut self, (): Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

/// Context for the [`ExtractNotesSuperState`].
///
/// Holds the SEC client of type `C`, the `execution_id`, and the slot for the stored archive. The
/// transition into [`DownloadNotesArchive`] hands a clone of the client and the `execution_id` to
/// that state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractNotesSuperStateContext<C> {
    pub(crate) sec_client: C,
    pub(crate) execution_id: Uuid,
    pub(crate) archive_slot: ArchiveSlot,
}

impl<C> ExtractNotesSuperStateContext<C> {
    /// Creates a new [`ExtractNotesSuperStateContext`] from the SEC client and the
    /// `execution_id`.
    #[must_use]
    pub fn new(sec_client: C, execution_id: Uuid) -> Self {
        Self {
            sec_client,
            execution_id,
            archive_slot: ArchiveSlot::default(),
        }
    }
}

/// A shared slot that [`StoreNotesArchive`] fills with the [`NotesArchiveRef`] of the stored file.
///
/// Clones share one slot. Every slot compares equal and adds nothing to a hash, so the slot never
/// changes how two contexts compare.
#[derive(Debug, Clone, Default)]
pub(crate) struct ArchiveSlot(Arc<Mutex<Option<NotesArchiveRef>>>);

impl ArchiveSlot {
    fn put(&self, archive: NotesArchiveRef) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(archive);
    }

    fn take(&self) -> Option<NotesArchiveRef> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).take()
    }
}

impl PartialEq for ArchiveSlot {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for ArchiveSlot {}

impl PartialOrd for ArchiveSlot {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ArchiveSlot {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl Hash for ArchiveSlot {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

impl<C: SecClientTrait> SMContext for ExtractNotesSuperStateContext<C> {
    type UpdateType = ();
    fn context(&self) -> &Self {
        self
    }
    fn update_context(&mut self, (): Self::UpdateType) {}
}

impl<C: SecClientTrait> Context for ExtractNotesSuperStateContext<C> {
    fn max_retries(&self) -> u32 {
        0
    }
}

/// Hierarchical super-state that extracts one Notes file.
///
/// Wraps the active state in its `S` type parameter and owns the SEC client of type `C` and the
/// `execution_id`. It moves through type-safe transitions: [`ValidateNotesPeriod`] →
/// [`PrepareNotesRequest`] → [`DownloadNotesArchive`] → [`StoreNotesArchive`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractNotesSuperState<S: State, C: SecClientTrait = SecClient> {
    current_state: S,
    input: ExtractNotesSuperStateData,
    output: Option<ExtractNotesSuperStateData>,
    context: ExtractNotesSuperStateContext<C>,
}

impl<S: State, C: SecClientTrait> Display for ExtractNotesSuperState<S, C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Extract Notes SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }
}

impl<S: State, C: SecClientTrait> SMState for ExtractNotesSuperState<S, C> {
    type InputData = ExtractNotesSuperStateData;
    type OutputData = ExtractNotesSuperStateData;
    type Context = ExtractNotesSuperStateContext<C>;

    fn state_name(&self) -> impl ToString {
        self.to_string()
    }
    fn input_data(&self) -> &Self::InputData {
        &self.input
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
            tokio::task::block_in_place(|| {
                handle.block_on(self.current_state.compute_output_data_async())
            })
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.current_state.compute_output_data_async())
        };

        if let Err(e) = result {
            let state_err: StateError = e.into();
            panic!("compute_output_data failed: {state_err}")
        }
    }
    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
    fn context_data(&self) -> &Self::Context {
        &self.context
    }
}

#[async_trait]
impl<S: State, C: SecClientTrait> State for ExtractNotesSuperState<S, C> {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.current_state
            .compute_output_data_async()
            .await
            .map_err(Into::into)
    }
}

impl<S: State, C: SecClientTrait> SMStateMachine<S> for ExtractNotesSuperState<S, C> {
    fn current_state(&self) -> &S {
        &self.current_state
    }
    fn current_state_mut(&mut self) -> &mut S {
        &mut self.current_state
    }
    fn run(&mut self) {
        unimplemented!(
            "SEC state machines are async-only. \
             Use into_stream or drive states manually"
        )
    }
    fn advance_state(&mut self) {
        unimplemented!(
            "SEC state machines are async-only. \
             Use into_stream or drive states manually"
        )
    }
}

impl<S: State, C: SecClientTrait> StateMachine<S> for ExtractNotesSuperState<S, C> {}

impl<S: State, C: SecClientTrait> SMSuperState<S> for ExtractNotesSuperState<S, C> {}

impl<S: State, C: SecClientTrait> SuperState<S> for ExtractNotesSuperState<S, C> {}

impl<R: NotesRepository> ExtractNotesSuperState<ValidateNotesPeriod<R>> {
    /// Returns an [`ExtractNotesSuperStateBuilder`] that holds no field yet.
    #[must_use]
    pub const fn builder()
    -> ExtractNotesSuperStateBuilder<NoPeriod, NoSecClient, NoExecutionId, NoRepository<R>> {
        ExtractNotesSuperStateBuilder::new()
    }
}

// --- Streaming ---

impl<S: State, C: SecClientTrait> ExtractNotesSuperState<S, C>
where
    Self: IntoStateMachineStream,
{
    /// Returns the stream, tagged with the `execution_id` that the context holds.
    ///
    /// Use it instead of `into_stream`, so the stream and the stored document name the same run.
    #[must_use]
    pub fn into_run_stream(self) -> StateMachineStream {
        let execution_id = self.context.execution_id;
        self.into_stream(execution_id)
    }

    /// Runs the stream to its end and returns the [`NotesArchiveRef`] of the stored file.
    ///
    /// Calls `on_item` with each successful stream item, in stream order. Each call gets its own
    /// slot for the archive, so clones of one super-state never share it.
    ///
    /// # Errors
    ///
    /// Returns the [`StreamError`] of the first state that fails. The stream ends at that error.
    ///
    /// # Panics
    ///
    /// Panics if the stream ends without an error before [`StoreNotesArchive`] computes its
    /// output. The state flow rules this out.
    pub async fn run_to_archive(
        mut self,
        mut on_item: impl FnMut(&StreamItem),
    ) -> Result<NotesArchiveRef, StreamError> {
        let archive_slot = ArchiveSlot::default();
        self.context.archive_slot = archive_slot.clone();
        let mut stream = self.into_run_stream();
        while let Some(item) = stream.next().await {
            on_item(&item?);
        }
        Ok(archive_slot.take().expect(
            "Given a stream that ended without an error, StoreNotesArchive has always put the archive into the slot",
        ))
    }
}

impl<R: NotesRepository, C: SecClientTrait> NonTerminal
    for ExtractNotesSuperState<ValidateNotesPeriod<R>, C>
{
    type Current = ValidateNotesPeriod<R>;
    type Next = PrepareNotesRequest<R>;
}

// Deviation: the impls that reach `DownloadNotesArchive` pin the response and the error of `C`.
// See the comment on `State for DownloadNotesArchive`.
impl<R, C> NonTerminal for ExtractNotesSuperState<PrepareNotesRequest<R>, C>
where
    R: NotesRepository,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type Current = PrepareNotesRequest<R>;
    type Next = DownloadNotesArchive<R, C>;
}

impl<R, C> NonTerminal for ExtractNotesSuperState<DownloadNotesArchive<R, C>, C>
where
    R: NotesRepository,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type Current = DownloadNotesArchive<R, C>;
    type Next = StoreNotesArchive<R>;
}

/// Ends the stream after [`StoreNotesArchive`] writes the Notes file to the store.
impl<R: NotesRepository + 'static, C: SecClientTrait + 'static> IntoStateMachineStream
    for ExtractNotesSuperState<StoreNotesArchive<R>, C>
{
    fn into_stream(self, execution_id: Uuid) -> StateMachineStream {
        let archive_slot = self.context.archive_slot.clone();
        terminal_stream_with(self, execution_id, move |state: &StoreNotesArchive<R>| {
            if let Some(output) = state.output_data() {
                archive_slot.put(output.archive.clone());
            }
        })
    }
}

// --- Transitions ---

impl<R: NotesRepository, C: SecClientTrait>
    Transition<ValidateNotesPeriod<R>, PrepareNotesRequest<R>>
    for ExtractNotesSuperState<ValidateNotesPeriod<R>, C>
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        Ok(ExtractNotesSuperState {
            current_state: PrepareNotesRequest::try_from(self.current_state)?,
            input: ExtractNotesSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R: NotesRepository, C: SecClientTrait>
    SMTransition<ValidateNotesPeriod<R>, PrepareNotesRequest<R>>
    for ExtractNotesSuperState<ValidateNotesPeriod<R>, C>
{
    type NewStateMachine = ExtractNotesSuperState<PrepareNotesRequest<R>, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<R, C> Transition<PrepareNotesRequest<R>, DownloadNotesArchive<R, C>>
    for ExtractNotesSuperState<PrepareNotesRequest<R>, C>
where
    R: NotesRepository,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let sec_client = self.context.sec_client.clone();
        let execution_id = self.context.execution_id;
        Ok(ExtractNotesSuperState {
            current_state: DownloadNotesArchive::try_from((
                self.current_state,
                sec_client,
                execution_id,
            ))?,
            input: ExtractNotesSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R, C> SMTransition<PrepareNotesRequest<R>, DownloadNotesArchive<R, C>>
    for ExtractNotesSuperState<PrepareNotesRequest<R>, C>
where
    R: NotesRepository,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type NewStateMachine = ExtractNotesSuperState<DownloadNotesArchive<R, C>, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<R, C> Transition<DownloadNotesArchive<R, C>, StoreNotesArchive<R>>
    for ExtractNotesSuperState<DownloadNotesArchive<R, C>, C>
where
    R: NotesRepository,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        Ok(ExtractNotesSuperState {
            current_state: StoreNotesArchive::try_from(self.current_state)?,
            input: ExtractNotesSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R, C> SMTransition<DownloadNotesArchive<R, C>, StoreNotesArchive<R>>
    for ExtractNotesSuperState<DownloadNotesArchive<R, C>, C>
where
    R: NotesRepository,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type NewStateMachine = ExtractNotesSuperState<StoreNotesArchive<R>, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use futures_util::StreamExt;
    use pretty_assertions::assert_eq;
    use storage::implementations::repository::filesystem::{
        DocumentKey, FilesystemRepository, Write,
    };
    use storage::{Sha256Digest, WriteError};
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::states::extract::download_notes_archive::constants::STATE_NAME as DOWNLOAD_NOTES_ARCHIVE;
    use crate::implementations::states::extract::store_notes_archive::constants::STATE_NAME as STORE_NOTES_ARCHIVE;
    use crate::implementations::states::extract::validate_notes_period::constants::STATE_NAME as VALIDATE_NOTES_PERIOD;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::raw_document::RawDocument;
    use crate::shared::raw_source::RawNotesArchive;
    use crate::shared::request::implementations::sec_request::{NotesFile, RequestKind};
    use crate::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_write_repository::FakeNotesRepository;

    type NotesFilesystemRepository = FilesystemRepository<Write, RawDocument<RawNotesArchive>>;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);
    const PREPARE_NOTES_REQUEST: &str = NotesFile::PREPARE_STATE_NAME;

    fn sec_client() -> AlwaysRespondingSecClient {
        AlwaysRespondingSecClient::new(
            200,
            vec![("content-type", "application/zip".to_string())],
            SAMPLE_NOTES_ZIP.to_vec(),
        )
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn filesystem_repository(root: &Path) -> NotesFilesystemRepository {
        FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<RawNotesArchive>>()
            .root(root)
            .build()
    }

    fn super_state<R: NotesRepository>(
        raw_period: &str,
        repository: R,
    ) -> ExtractNotesSuperState<ValidateNotesPeriod<R>, AlwaysRespondingSecClient> {
        ExtractNotesSuperState::builder()
            .period(raw_period)
            .sec_client(sec_client())
            .execution_id(SAMPLE_EXECUTION_ID)
            .repository(repository)
            .build()
    }

    /// Runs the whole stream and returns its items, or panics on the first error.
    async fn run_stream<R: NotesRepository + 'static>(
        super_state: ExtractNotesSuperState<ValidateNotesPeriod<R>, AlwaysRespondingSecClient>,
    ) -> Vec<StreamItem> {
        super_state
            .into_run_stream()
            .map(|item| {
                item.expect(
                    "Given a published period, the fake client, and a writable store, every stream item should be a success",
                )
            })
            .collect()
            .await
    }

    /// Drives the super-state from the builder to `DownloadNotesArchive` with a computed output.
    async fn downloaded_super_state() -> ExtractNotesSuperState<
        DownloadNotesArchive<FakeNotesRepository, AlwaysRespondingSecClient>,
        AlwaysRespondingSecClient,
    > {
        let mut validate = super_state("2024q1", FakeNotesRepository::new());
        validate
            .compute_output_data_async()
            .await
            .expect("Given a published period, validation should always succeed");
        let mut prepare = validate
            .transition_to_next_state_sec()
            .expect("Given a computed validation, the transition should always succeed");
        prepare
            .compute_output_data_async()
            .await
            .expect("Given a validated period, preparing the request should always succeed");
        let mut download = prepare
            .transition_to_next_state_sec()
            .expect("Given a prepared request, the transition should always succeed");
        download.compute_output_data_async().await.expect(
            "Given the fake client with the ZIP fixture, the download should always succeed",
        );
        download
    }

    #[test]
    #[expect(
        clippy::redundant_clone,
        reason = "False positive: `state_name` returns `impl ToString`, which is a `String` here. Clippy flags `.to_string()` as a clone, but removing it does not compile"
    )]
    fn should_return_super_state_name_with_current_state_when_in_validate_notes_period_state() {
        let super_state = super_state("2024q1", FakeNotesRepository::new());

        let expected_result = "Extract Notes SuperState (Current: Validate Notes Period)";

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_zip_fixture_under_sec_notes_when_running_the_stream() {
        let root = temporary_root();
        let super_state = super_state("2024q1", filesystem_repository(root.path()));

        let expected_result = SAMPLE_NOTES_ZIP.to_vec();

        run_stream(super_state).await;
        let result = std::fs::read(root.path().join("sec/notes/2024q1_notes.zip")).expect(
            "Given a stream that completed without an error, the stored file should always exist",
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_the_events_of_every_state_in_order_when_running_the_stream() {
        let super_state = super_state("2024q1", FakeNotesRepository::new());

        let expected_result = vec![
            (StreamEvent::StateStarted, VALIDATE_NOTES_PERIOD),
            (StreamEvent::StateCompleted, VALIDATE_NOTES_PERIOD),
            (StreamEvent::TransitionCompleted, VALIDATE_NOTES_PERIOD),
            (StreamEvent::StateStarted, PREPARE_NOTES_REQUEST),
            (StreamEvent::StateCompleted, PREPARE_NOTES_REQUEST),
            (StreamEvent::TransitionCompleted, PREPARE_NOTES_REQUEST),
            (StreamEvent::StateStarted, DOWNLOAD_NOTES_ARCHIVE),
            (StreamEvent::StateCompleted, DOWNLOAD_NOTES_ARCHIVE),
            (StreamEvent::TransitionCompleted, DOWNLOAD_NOTES_ARCHIVE),
            (StreamEvent::StateStarted, STORE_NOTES_ARCHIVE),
            (StreamEvent::StateCompleted, STORE_NOTES_ARCHIVE),
        ];

        let items = run_stream(super_state).await;
        let result: Vec<(StreamEvent, &str)> = items
            .iter()
            .map(|item| (item.event.clone(), item.state_name.as_str()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_end_the_stream_with_the_notes_archive_ref_when_running_the_stream() {
        let super_state = super_state("2024q1", FakeNotesRepository::new());
        let period = NotesPeriod::new("2024q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed");
        let key = DocumentKey::new("sec/notes/2024q1_notes.zip")
            .expect("Given a hardcoded valid relative path, the key should always build");
        let archive = NotesArchiveRef::new(
            period,
            key,
            SAMPLE_NOTES_ZIP.len() as u64,
            Sha256Digest::of(SAMPLE_NOTES_ZIP),
        );

        let expected_result = Some(serde_json::json!({ "archive": archive }));

        let items = run_stream(super_state).await;
        let result = items.last().map(|item| item.data["output"].clone());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_hand_store_notes_archive_the_only_reference_to_the_response_when_transitioning()
    {
        let download = downloaded_super_state().await;

        let expected_result = Some(1);

        let store = download
            .transition_to_next_state_sec()
            .expect("Given a computed download, the transition should always succeed");
        let result = store
            .current_state()
            .input_data()
            .response
            .as_ref()
            .map(Arc::strong_count);

        assert_eq!(result, expected_result);
    }

    /// Runs the whole stream and returns the state names of its errors.
    async fn failed_state_names<R: NotesRepository + 'static>(
        super_state: ExtractNotesSuperState<ValidateNotesPeriod<R>, AlwaysRespondingSecClient>,
    ) -> Vec<String> {
        super_state
            .into_run_stream()
            .filter_map(|item| async move { item.err().map(|error| error.state_name) })
            .collect()
            .await
    }

    #[tokio::test]
    async fn should_end_the_stream_with_a_failed_download_when_the_notes_file_is_not_found() {
        let super_state = ExtractNotesSuperState::builder()
            .period("2024q1")
            .sec_client(AlwaysRespondingSecClient::new(404, Vec::new(), Vec::new()))
            .execution_id(SAMPLE_EXECUTION_ID)
            .repository(FakeNotesRepository::new())
            .build();

        let expected_result = vec![DOWNLOAD_NOTES_ARCHIVE.to_string()];

        let result = failed_state_names(super_state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_end_the_stream_with_a_failed_store_when_the_repository_rejects_the_write() {
        let repository =
            FakeNotesRepository::rejecting(WriteError::conflicting_write("Sample conflict"));
        let super_state = super_state("2024q1", repository);

        let expected_result = vec![STORE_NOTES_ARCHIVE.to_string()];

        let result = failed_state_names(super_state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_serialize_a_null_response_when_store_notes_archive_took_the_body() {
        let super_state = super_state("2024q1", FakeNotesRepository::new());

        let expected_result = Some(serde_json::Value::Null);

        let items = run_stream(super_state).await;
        let result = items
            .last()
            .and_then(|item| item.data.pointer("/input/response").cloned());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_tag_stream_with_execution_id_of_context_when_streaming_the_run() {
        let super_state = super_state("2024q5", FakeNotesRepository::new());

        let expected_result = vec![SAMPLE_EXECUTION_ID];

        let result: Vec<Uuid> = super_state
            .into_run_stream()
            .filter_map(|item| async move { item.err().map(|error| error.execution_id) })
            .collect()
            .await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_notes_archive_ref_when_running_to_the_archive() {
        let super_state = super_state("2024q1", FakeNotesRepository::new());
        let period = NotesPeriod::new("2024q1")
            .expect("Given a hardcoded published quarter, parsing should always succeed");
        let key = DocumentKey::new("sec/notes/2024q1_notes.zip")
            .expect("Given a hardcoded valid relative path, the key should always build");

        let expected_result = Some(NotesArchiveRef::new(
            period,
            key,
            SAMPLE_NOTES_ZIP.len() as u64,
            Sha256Digest::of(SAMPLE_NOTES_ZIP),
        ));

        let result = super_state.run_to_archive(|_| {}).await.ok();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_error_of_the_failed_state_when_running_to_the_archive() {
        let repository =
            FakeNotesRepository::rejecting(WriteError::conflicting_write("Sample conflict"));
        let super_state = super_state("2024q1", repository);

        let expected_result = Some(STORE_NOTES_ARCHIVE.to_string());

        let result = super_state
            .run_to_archive(|_| {})
            .await
            .err()
            .map(|error| error.state_name);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_hand_every_stream_item_to_the_caller_when_running_to_the_archive() {
        let archive_run = super_state("2024q1", FakeNotesRepository::new());

        let expected_result: Vec<(StreamEvent, String)> =
            run_stream(super_state("2024q1", FakeNotesRepository::new()))
                .await
                .into_iter()
                .map(|item| (item.event, item.state_name))
                .collect();

        let mut result = Vec::new();
        archive_run
            .run_to_archive(|item| result.push((item.event.clone(), item.state_name.clone())))
            .await
            .expect("Given a published period, the fake client, and a fake store, the run should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_archive_when_a_clone_empties_the_slot_it_was_built_with() {
        let super_state = super_state("2024q1", FakeNotesRepository::new());
        let built_slot = super_state.context.archive_slot.clone();

        let expected_result = Some(PathBuf::from("sec/notes/2024q1_notes.zip"));

        let result = super_state
            .run_to_archive(|_| {
                built_slot.take();
            })
            .await
            .ok()
            .map(|archive| archive.key().as_path().to_path_buf());

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_store_notes_archive_super_state() {
        implements_auto_traits::<ExtractNotesSuperState<StoreNotesArchive<FakeNotesRepository>>>();
    }
}
