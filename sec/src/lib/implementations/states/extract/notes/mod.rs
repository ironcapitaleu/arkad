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
//! [`StoreNotesArchive`]. The stream ends after [`StoreNotesArchive`]. Its last item holds the
//! [`NotesArchiveRef`](crate::shared::notes_archive_ref::NotesArchiveRef) in the output data.
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

use std::fmt::{self, Display, Formatter};

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::{StateMachine as SMStateMachine, Transition as SMTransition};
use uuid::Uuid;

use crate::error::State as StateError;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::download_notes_archive::{
    DownloadNotesArchive, NotesFileClient,
};
use crate::implementations::states::extract::notes::builder::{
    ExtractNotesSuperStateBuilder, NoExecutionId, NoPeriod, NoRepository, NoSecClient,
};
use crate::implementations::states::extract::notes_repository::NotesRepository;
use crate::implementations::states::extract::prepare_notes_request::PrepareNotesRequest;
use crate::implementations::states::extract::store_notes_archive::StoreNotesArchive;
use crate::implementations::states::extract::validate_notes_period::ValidateNotesPeriod;
use crate::prelude::*;
use crate::shared::http_client::implementations::sec_client::SecClient;

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
/// Holds the SEC client of type `C` and the `execution_id`. The transition into
/// [`DownloadNotesArchive`] hands a clone of the client to that state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractNotesSuperStateContext<C> {
    pub(crate) sec_client: C,
    pub(crate) execution_id: Uuid,
}

impl<C> ExtractNotesSuperStateContext<C> {
    /// Creates a new [`ExtractNotesSuperStateContext`] from the SEC client and the
    /// `execution_id`.
    #[must_use]
    pub const fn new(sec_client: C, execution_id: Uuid) -> Self {
        Self {
            sec_client,
            execution_id,
        }
    }
}

impl<C: NotesFileClient> SMContext for ExtractNotesSuperStateContext<C> {
    type UpdateType = ();
    fn context(&self) -> &Self {
        self
    }
    fn update_context(&mut self, (): Self::UpdateType) {}
}

impl<C: NotesFileClient> Context for ExtractNotesSuperStateContext<C> {
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
pub struct ExtractNotesSuperState<S: State, C: NotesFileClient = SecClient> {
    current_state: S,
    input: ExtractNotesSuperStateData,
    output: Option<ExtractNotesSuperStateData>,
    context: ExtractNotesSuperStateContext<C>,
}

impl<S: State, C: NotesFileClient> Display for ExtractNotesSuperState<S, C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Extract Notes SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }
}

impl<S: State, C: NotesFileClient> SMState for ExtractNotesSuperState<S, C> {
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
impl<S: State, C: NotesFileClient> State for ExtractNotesSuperState<S, C> {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.current_state
            .compute_output_data_async()
            .await
            .map_err(Into::into)
    }
}

impl<S: State, C: NotesFileClient> SMStateMachine<S> for ExtractNotesSuperState<S, C> {
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

impl<S: State, C: NotesFileClient> StateMachine<S> for ExtractNotesSuperState<S, C> {}

impl<S: State, C: NotesFileClient> SMSuperState<S> for ExtractNotesSuperState<S, C> {}

impl<S: State, C: NotesFileClient> SuperState<S> for ExtractNotesSuperState<S, C> {}

impl<R: NotesRepository> ExtractNotesSuperState<ValidateNotesPeriod<R>> {
    /// Returns an [`ExtractNotesSuperStateBuilder`] that holds no field yet.
    #[must_use]
    pub const fn builder()
    -> ExtractNotesSuperStateBuilder<NoPeriod, NoSecClient, NoExecutionId, NoRepository<R>> {
        ExtractNotesSuperStateBuilder::new()
    }
}

// --- Streaming ---

impl<S: State, C: NotesFileClient> ExtractNotesSuperState<S, C>
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
}

impl<R: NotesRepository, C: NotesFileClient> NonTerminal
    for ExtractNotesSuperState<ValidateNotesPeriod<R>, C>
{
    type Current = ValidateNotesPeriod<R>;
    type Next = PrepareNotesRequest<R>;
}

impl<R: NotesRepository, C: NotesFileClient> NonTerminal
    for ExtractNotesSuperState<PrepareNotesRequest<R>, C>
{
    type Current = PrepareNotesRequest<R>;
    type Next = DownloadNotesArchive<R, C>;
}

impl<R: NotesRepository, C: NotesFileClient> NonTerminal
    for ExtractNotesSuperState<DownloadNotesArchive<R, C>, C>
{
    type Current = DownloadNotesArchive<R, C>;
    type Next = StoreNotesArchive<R>;
}

/// Ends the stream after [`StoreNotesArchive`] writes the Notes file to the store.
impl<R: NotesRepository + 'static, C: NotesFileClient + 'static> IntoStateMachineStream
    for ExtractNotesSuperState<StoreNotesArchive<R>, C>
{
    fn into_stream(self, execution_id: Uuid) -> StateMachineStream {
        terminal_stream(self, execution_id)
    }
}

// --- Transitions ---

impl<R: NotesRepository, C: NotesFileClient>
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

impl<R: NotesRepository, C: NotesFileClient>
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

impl<R: NotesRepository, C: NotesFileClient>
    Transition<PrepareNotesRequest<R>, DownloadNotesArchive<R, C>>
    for ExtractNotesSuperState<PrepareNotesRequest<R>, C>
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let sec_client = self.context.sec_client.clone();
        Ok(ExtractNotesSuperState {
            current_state: DownloadNotesArchive::try_from((self.current_state, sec_client))?,
            input: ExtractNotesSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R: NotesRepository, C: NotesFileClient>
    SMTransition<PrepareNotesRequest<R>, DownloadNotesArchive<R, C>>
    for ExtractNotesSuperState<PrepareNotesRequest<R>, C>
{
    type NewStateMachine = ExtractNotesSuperState<DownloadNotesArchive<R, C>, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<R: NotesRepository, C: NotesFileClient>
    Transition<DownloadNotesArchive<R, C>, StoreNotesArchive<R>>
    for ExtractNotesSuperState<DownloadNotesArchive<R, C>, C>
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

impl<R: NotesRepository, C: NotesFileClient>
    SMTransition<DownloadNotesArchive<R, C>, StoreNotesArchive<R>>
    for ExtractNotesSuperState<DownloadNotesArchive<R, C>, C>
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
    use std::path::Path;
    use std::sync::Arc;

    use futures_util::StreamExt;
    use pretty_assertions::assert_eq;
    use storage::Sha256Digest;
    use storage::implementations::repository::filesystem::{
        DocumentKey, FilesystemRepository, Write,
    };
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::states::extract::download_notes_archive::constants::STATE_NAME as DOWNLOAD_NOTES_ARCHIVE;
    use crate::implementations::states::extract::prepare_notes_request::constants::STATE_NAME as PREPARE_NOTES_REQUEST;
    use crate::implementations::states::extract::store_notes_archive::constants::STATE_NAME as STORE_NOTES_ARCHIVE;
    use crate::implementations::states::extract::validate_notes_period::constants::STATE_NAME as VALIDATE_NOTES_PERIOD;
    use crate::shared::notes_archive_ref::NotesArchiveRef;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::raw_document::RawDocument;
    use crate::shared::raw_source::RawNotesArchive;
    use crate::tests::fixtures::data::notes_archive::SAMPLE_NOTES_ZIP;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingReadySecClient;
    use crate::tests::fixtures::sample_write_repository::FakeNotesRepository;

    type NotesFilesystemRepository = FilesystemRepository<Write, RawDocument<RawNotesArchive>>;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);

    fn sec_client() -> AlwaysRespondingReadySecClient {
        AlwaysRespondingReadySecClient::new(
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
    ) -> ExtractNotesSuperState<ValidateNotesPeriod<R>, AlwaysRespondingReadySecClient> {
        ExtractNotesSuperState::builder()
            .period(raw_period)
            .sec_client(sec_client())
            .execution_id(SAMPLE_EXECUTION_ID)
            .repository(repository)
            .build()
    }

    /// Runs the whole stream and returns its items, or panics on the first error.
    async fn run_stream<R: NotesRepository + 'static>(
        super_state: ExtractNotesSuperState<ValidateNotesPeriod<R>, AlwaysRespondingReadySecClient>,
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
        DownloadNotesArchive<FakeNotesRepository, AlwaysRespondingReadySecClient>,
        AlwaysRespondingReadySecClient,
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

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_store_notes_archive_super_state() {
        implements_auto_traits::<ExtractNotesSuperState<StoreNotesArchive<FakeNotesRepository>>>();
    }
}
