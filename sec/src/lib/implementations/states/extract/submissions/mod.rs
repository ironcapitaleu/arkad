//! # Extract Submissions Super-State
//!
//! Provides the [`ExtractSubmissionsSuperState`], which fetches the `submissions` JSON of one
//! company and writes it to the store.
//!
//! The super-state owns the shared SEC client, the `execution_id`, the ingestion mode of the run,
//! and the storage repository. The transition into [`StoreSubmissions`] passes a clone of the
//! repository to the store state. Both ingestion paths run this machine, so the caller sets the
//! mode, and the metadata of the stored document records it.
//!
//! ## Modules
//!
//! - [`builder`]: Builds the [`ExtractSubmissionsSuperState`] at its entry state
//!   [`ValidateCikFormat`].
//!
//! ## State Flow
//!
//! [`ValidateCikFormat`] → [`PrepareSecRequest`] of the [`Submissions`] kind →
//! [`ExecuteSecRequest`] of the [`Submissions`] kind → [`StoreSubmissions`]. The stream ends after
//! [`StoreSubmissions`].
//!
//! ## Usage
//!
//! ```no_run
//! use futures_util::StreamExt;
//! use sec::implementations::states::extract::submissions::ExtractSubmissionsSuperState;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use sec::shared::raw_document::RawDocument;
//! use sec::shared::raw_source::RawSubmissions;
//! use storage::implementations::repository::filesystem::{FilesystemRepository, IngestionMode};
//! use uuid::Uuid;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let repository = FilesystemRepository::builder()
//!     .writer()
//!     .document_type::<RawDocument<RawSubmissions>>()
//!     .root("data")
//!     .build();
//! let extract_state = ExtractSubmissionsSuperState::builder()
//!     .cik("1067983")
//!     .sec_client(SecClient::default())
//!     .execution_id(Uuid::new_v4())
//!     .ingestion_mode(IngestionMode::Backfill)
//!     .repository(repository)
//!     .build();
//! let mut stream = extract_state.into_run_stream();
//! while let Some(item) = stream.next().await {
//!     let _item = item.expect("The live SEC fetch should succeed");
//! }
//! # }
//! ```

pub mod builder;

use std::fmt::{self, Display, Formatter};

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::{StateMachine as SMStateMachine, Transition as SMTransition};
use storage::implementations::repository::filesystem::IngestionMode;
use uuid::Uuid;

use crate::error::State as StateError;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest;
use crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
use crate::implementations::states::extract::store_submissions::StoreSubmissions;
use crate::implementations::states::extract::submissions::builder::{
    ExtractSubmissionsSuperStateBuilder, NoCik, NoExecutionId, NoIngestionMode, NoRepository,
    NoSecClient,
};
use crate::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use crate::prelude::*;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawSubmissions;
use crate::shared::request::implementations::sec_request::Submissions;
use crate::shared::response::implementations::sec_response::{JsonBody, SecResponse};
use crate::traits::state_machine::stream::terminal_stream;

/// Input and output data for the [`ExtractSubmissionsSuperState`].
///
/// A placeholder with unit update semantics: the super-state delegates all real data handling to
/// its current inner state, so it carries no data of its own.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ExtractSubmissionsSuperStateData;

impl SMStateData for ExtractSubmissionsSuperStateData {
    type UpdateType = ();
    fn state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, (): Self::UpdateType) {}
}

impl StateData for ExtractSubmissionsSuperStateData {
    fn update_state(&mut self, (): Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

/// Context for the [`ExtractSubmissionsSuperState`].
///
/// Holds the SEC client of type `C`, the `execution_id`, the ingestion mode, and the storage
/// repository `R`. The transition into [`ExecuteSecRequest`] passes a clone of the client to that
/// state. The transition into [`StoreSubmissions`] passes the `execution_id`, the ingestion mode,
/// and a clone of the repository to the store state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractSubmissionsSuperStateContext<R, C> {
    pub(crate) sec_client: C,
    pub(crate) execution_id: Uuid,
    pub(crate) ingestion_mode: IngestionMode,
    pub(crate) repository: R,
}

impl<R, C> ExtractSubmissionsSuperStateContext<R, C> {
    /// Creates a new [`ExtractSubmissionsSuperStateContext`] from the SEC client, the
    /// `execution_id`, the ingestion mode, and the storage repository.
    #[must_use]
    pub const fn new(
        sec_client: C,
        execution_id: Uuid,
        ingestion_mode: IngestionMode,
        repository: R,
    ) -> Self {
        Self {
            sec_client,
            execution_id,
            ingestion_mode,
            repository,
        }
    }
}

impl<R: RawRepository<RawSubmissions>, C: SecClientTrait> SMContext
    for ExtractSubmissionsSuperStateContext<R, C>
{
    type UpdateType = ();
    fn context(&self) -> &Self {
        self
    }
    fn update_context(&mut self, (): Self::UpdateType) {}
}

impl<R: RawRepository<RawSubmissions>, C: SecClientTrait> Context
    for ExtractSubmissionsSuperStateContext<R, C>
{
    fn max_retries(&self) -> u32 {
        0
    }
}

/// Hierarchical super-state that extracts the `submissions` JSON of one company.
///
/// Wraps the active state in its `S` type parameter and owns the SEC client of type `C`, the
/// `execution_id`, the ingestion mode, and the storage repository `R`. It moves through type-safe transitions:
/// [`ValidateCikFormat`] → [`PrepareSecRequest`] → [`ExecuteSecRequest`] → [`StoreSubmissions`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractSubmissionsSuperState<
    S: State,
    R: RawRepository<RawSubmissions>,
    C: SecClientTrait = SecClient,
> {
    current_state: S,
    input: ExtractSubmissionsSuperStateData,
    output: Option<ExtractSubmissionsSuperStateData>,
    context: ExtractSubmissionsSuperStateContext<R, C>,
}

impl<S: State, R: RawRepository<RawSubmissions>, C: SecClientTrait> Display
    for ExtractSubmissionsSuperState<S, R, C>
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Extract Submissions SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }
}

impl<S: State, R: RawRepository<RawSubmissions>, C: SecClientTrait> SMState
    for ExtractSubmissionsSuperState<S, R, C>
{
    type InputData = ExtractSubmissionsSuperStateData;
    type OutputData = ExtractSubmissionsSuperStateData;
    type Context = ExtractSubmissionsSuperStateContext<R, C>;

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
impl<S: State, R: RawRepository<RawSubmissions>, C: SecClientTrait> State
    for ExtractSubmissionsSuperState<S, R, C>
{
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.current_state
            .compute_output_data_async()
            .await
            .map_err(Into::into)
    }
}

impl<S: State, R: RawRepository<RawSubmissions>, C: SecClientTrait> SMStateMachine<S>
    for ExtractSubmissionsSuperState<S, R, C>
{
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

impl<S: State, R: RawRepository<RawSubmissions>, C: SecClientTrait> StateMachine<S>
    for ExtractSubmissionsSuperState<S, R, C>
{
}

impl<S: State, R: RawRepository<RawSubmissions>, C: SecClientTrait> SMSuperState<S>
    for ExtractSubmissionsSuperState<S, R, C>
{
}

impl<S: State, R: RawRepository<RawSubmissions>, C: SecClientTrait> SuperState<S>
    for ExtractSubmissionsSuperState<S, R, C>
{
}

impl<R: RawRepository<RawSubmissions>> ExtractSubmissionsSuperState<ValidateCikFormat, R> {
    /// Returns an [`ExtractSubmissionsSuperStateBuilder`] that holds no field yet.
    #[must_use]
    pub const fn builder() -> ExtractSubmissionsSuperStateBuilder<
        NoCik,
        NoSecClient,
        NoExecutionId,
        NoIngestionMode,
        NoRepository<R>,
    > {
        ExtractSubmissionsSuperStateBuilder::new()
    }
}

// --- Streaming ---

impl<S: State, R: RawRepository<RawSubmissions>, C: SecClientTrait>
    ExtractSubmissionsSuperState<S, R, C>
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

impl<R: RawRepository<RawSubmissions>, C: SecClientTrait> NonTerminal
    for ExtractSubmissionsSuperState<ValidateCikFormat, R, C>
{
    type Current = ValidateCikFormat;
    type Next = PrepareSecRequest<Submissions>;
}

// Deviation: the impls that reach `ExecuteSecRequest` pin the response and the error of `C`.
// See the comment on `State for ExecuteSecRequest`.
impl<R, C> NonTerminal for ExtractSubmissionsSuperState<PrepareSecRequest<Submissions>, R, C>
where
    R: RawRepository<RawSubmissions>,
    C: SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>, Error = FailedSecRequest>,
{
    type Current = PrepareSecRequest<Submissions>;
    type Next = ExecuteSecRequest<Submissions, C>;
}

impl<R, C> NonTerminal for ExtractSubmissionsSuperState<ExecuteSecRequest<Submissions, C>, R, C>
where
    R: RawRepository<RawSubmissions>,
    C: SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>, Error = FailedSecRequest>,
{
    type Current = ExecuteSecRequest<Submissions, C>;
    type Next = StoreSubmissions<R>;
}

/// Ends the stream after [`StoreSubmissions`] writes the `submissions` JSON to the store.
impl<R: RawRepository<RawSubmissions> + 'static, C: SecClientTrait + 'static> IntoStateMachineStream
    for ExtractSubmissionsSuperState<StoreSubmissions<R>, R, C>
{
    fn into_stream(self, execution_id: Uuid) -> StateMachineStream {
        terminal_stream(self, execution_id)
    }
}

// --- Transitions ---

impl<R: RawRepository<RawSubmissions>, C: SecClientTrait>
    Transition<ValidateCikFormat, PrepareSecRequest<Submissions>>
    for ExtractSubmissionsSuperState<ValidateCikFormat, R, C>
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        Ok(ExtractSubmissionsSuperState {
            current_state: PrepareSecRequest::try_from(self.current_state)?,
            input: ExtractSubmissionsSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R: RawRepository<RawSubmissions>, C: SecClientTrait>
    SMTransition<ValidateCikFormat, PrepareSecRequest<Submissions>>
    for ExtractSubmissionsSuperState<ValidateCikFormat, R, C>
{
    type NewStateMachine = ExtractSubmissionsSuperState<PrepareSecRequest<Submissions>, R, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<R, C> Transition<PrepareSecRequest<Submissions>, ExecuteSecRequest<Submissions, C>>
    for ExtractSubmissionsSuperState<PrepareSecRequest<Submissions>, R, C>
where
    R: RawRepository<RawSubmissions>,
    C: SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>, Error = FailedSecRequest>,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let sec_client = self.context.sec_client.clone();
        Ok(ExtractSubmissionsSuperState {
            current_state: ExecuteSecRequest::try_from((self.current_state, sec_client))?,
            input: ExtractSubmissionsSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R, C> SMTransition<PrepareSecRequest<Submissions>, ExecuteSecRequest<Submissions, C>>
    for ExtractSubmissionsSuperState<PrepareSecRequest<Submissions>, R, C>
where
    R: RawRepository<RawSubmissions>,
    C: SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>, Error = FailedSecRequest>,
{
    type NewStateMachine = ExtractSubmissionsSuperState<ExecuteSecRequest<Submissions, C>, R, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<R, C> Transition<ExecuteSecRequest<Submissions, C>, StoreSubmissions<R>>
    for ExtractSubmissionsSuperState<ExecuteSecRequest<Submissions, C>, R, C>
where
    R: RawRepository<RawSubmissions>,
    C: SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>, Error = FailedSecRequest>,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        Ok(ExtractSubmissionsSuperState {
            current_state: StoreSubmissions::try_from((
                self.current_state,
                self.context.execution_id,
                self.context.ingestion_mode,
                self.context.repository.clone(),
            ))?,
            input: ExtractSubmissionsSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R, C> SMTransition<ExecuteSecRequest<Submissions, C>, StoreSubmissions<R>>
    for ExtractSubmissionsSuperState<ExecuteSecRequest<Submissions, C>, R, C>
where
    R: RawRepository<RawSubmissions>,
    C: SecClientTrait<Response<JsonBody> = SecResponse<JsonBody>, Error = FailedSecRequest>,
{
    type NewStateMachine = ExtractSubmissionsSuperState<StoreSubmissions<R>, R, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use futures_util::StreamExt;
    use pretty_assertions::assert_eq;
    use storage::WriteError;
    use storage::implementations::repository::filesystem::{Document, FilesystemRepository, Write};
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::states::extract::execute_sec_request::constants::STATE_NAME as EXECUTE_SEC_REQUEST;
    use crate::implementations::states::extract::prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
    use crate::implementations::states::extract::store_submissions::constants::STATE_NAME as STORE_SUBMISSIONS;
    use crate::implementations::states::extract::validate_cik_format::constants::STATE_NAME as VALIDATE_CIK_FORMAT;
    use crate::shared::raw_document::RawDocument;
    use crate::tests::fixtures::data::submissions::SAMPLE_SUBMISSIONS_JSON;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::always_responding::AlwaysRespondingSecClient;
    use crate::tests::fixtures::sample_write_repository::FakeSubmissionsRepository;

    type SubmissionsFilesystemRepository = FilesystemRepository<Write, RawDocument<RawSubmissions>>;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);

    const SAMPLE_CIK: &str = "1067983";

    fn sec_client() -> AlwaysRespondingSecClient {
        AlwaysRespondingSecClient::new(
            200,
            vec![("content-type", "application/json".to_string())],
            SAMPLE_SUBMISSIONS_JSON.to_vec(),
        )
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn filesystem_repository(root: &Path) -> SubmissionsFilesystemRepository {
        FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<RawSubmissions>>()
            .root(root)
            .build()
    }

    fn super_state<R: RawRepository<RawSubmissions>>(
        raw_cik: &str,
        sec_client: AlwaysRespondingSecClient,
        ingestion_mode: IngestionMode,
        repository: R,
    ) -> ExtractSubmissionsSuperState<ValidateCikFormat, R, AlwaysRespondingSecClient> {
        ExtractSubmissionsSuperState::builder()
            .cik(raw_cik)
            .sec_client(sec_client)
            .execution_id(SAMPLE_EXECUTION_ID)
            .ingestion_mode(ingestion_mode)
            .repository(repository)
            .build()
    }

    /// Runs the whole stream and returns its items, or panics on the first error.
    async fn run_stream<R: RawRepository<RawSubmissions> + 'static>(
        super_state: ExtractSubmissionsSuperState<ValidateCikFormat, R, AlwaysRespondingSecClient>,
    ) -> Vec<StreamItem> {
        super_state
            .into_run_stream()
            .map(|item| {
                item.expect(
                    "Given a valid CIK, the fake client, and a writable store, every stream item should be a success",
                )
            })
            .collect()
            .await
    }

    /// Runs the whole stream and returns the state names of its errors.
    async fn failed_state_names<R: RawRepository<RawSubmissions> + 'static>(
        super_state: ExtractSubmissionsSuperState<ValidateCikFormat, R, AlwaysRespondingSecClient>,
    ) -> Vec<String> {
        super_state
            .into_run_stream()
            .filter_map(|item| async move { item.err().map(|error| error.state_name) })
            .collect()
            .await
    }

    #[test]
    #[expect(
        clippy::redundant_clone,
        reason = "False positive: `state_name` returns `impl ToString`, which is a `String` here. Clippy flags `.to_string()` as a clone, but removing it does not compile"
    )]
    fn should_return_super_state_name_with_current_state_when_in_validate_cik_format_state() {
        let super_state = super_state(
            SAMPLE_CIK,
            sec_client(),
            IngestionMode::Incremental,
            FakeSubmissionsRepository::new(),
        );

        let expected_result = "Extract Submissions SuperState (Current: Validate CIK Format)";

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_json_fixture_under_sec_submissions_when_running_the_stream() {
        let root = temporary_root();
        let super_state = super_state(
            SAMPLE_CIK,
            sec_client(),
            IngestionMode::Incremental,
            filesystem_repository(root.path()),
        );

        let expected_result = SAMPLE_SUBMISSIONS_JSON.to_vec();

        run_stream(super_state).await;
        let result = std::fs::read(root.path().join("sec/submissions/CIK0001067983.json")).expect(
            "Given a stream that completed without an error, the stored file should always exist",
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_the_events_of_every_state_in_order_when_running_the_stream() {
        let super_state = super_state(
            SAMPLE_CIK,
            sec_client(),
            IngestionMode::Incremental,
            FakeSubmissionsRepository::new(),
        );

        let expected_result = vec![
            (StreamEvent::StateStarted, VALIDATE_CIK_FORMAT),
            (StreamEvent::StateCompleted, VALIDATE_CIK_FORMAT),
            (StreamEvent::TransitionCompleted, VALIDATE_CIK_FORMAT),
            (StreamEvent::StateStarted, PREPARE_SEC_REQUEST),
            (StreamEvent::StateCompleted, PREPARE_SEC_REQUEST),
            (StreamEvent::TransitionCompleted, PREPARE_SEC_REQUEST),
            (StreamEvent::StateStarted, EXECUTE_SEC_REQUEST),
            (StreamEvent::StateCompleted, EXECUTE_SEC_REQUEST),
            (StreamEvent::TransitionCompleted, EXECUTE_SEC_REQUEST),
            (StreamEvent::StateStarted, STORE_SUBMISSIONS),
            (StreamEvent::StateCompleted, STORE_SUBMISSIONS),
        ];

        let items = run_stream(super_state).await;
        let result: Vec<(StreamEvent, &str)> = items
            .iter()
            .map(|item| (item.event.clone(), item.state_name.as_str()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_the_backfill_mode_of_the_builder_when_running_the_stream() {
        let ingestion_mode = IngestionMode::Backfill;
        let repository = FakeSubmissionsRepository::new();
        let super_state = super_state(SAMPLE_CIK, sec_client(), ingestion_mode, repository.clone());

        let expected_result = vec![IngestionMode::Backfill];

        run_stream(super_state).await;
        let result: Vec<IngestionMode> = repository
            .persisted()
            .iter()
            .map(|document| document.metadata().ingestion_mode())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_the_incremental_mode_of_the_builder_when_running_the_stream() {
        let ingestion_mode = IngestionMode::Incremental;
        let repository = FakeSubmissionsRepository::new();
        let super_state = super_state(SAMPLE_CIK, sec_client(), ingestion_mode, repository.clone());

        let expected_result = vec![IngestionMode::Incremental];

        run_stream(super_state).await;
        let result: Vec<IngestionMode> = repository
            .persisted()
            .iter()
            .map(|document| document.metadata().ingestion_mode())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_end_the_stream_with_a_failed_validation_when_the_cik_is_not_numeric() {
        let super_state = super_state(
            "BRK-B",
            sec_client(),
            IngestionMode::Incremental,
            FakeSubmissionsRepository::new(),
        );

        let expected_result = vec![VALIDATE_CIK_FORMAT.to_string()];

        let result = failed_state_names(super_state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_end_the_stream_with_a_failed_fetch_when_the_submissions_are_not_found() {
        let sec_client = AlwaysRespondingSecClient::new(404, Vec::new(), Vec::new());
        let super_state = super_state(
            SAMPLE_CIK,
            sec_client,
            IngestionMode::Incremental,
            FakeSubmissionsRepository::new(),
        );

        let expected_result = vec![EXECUTE_SEC_REQUEST.to_string()];

        let result = failed_state_names(super_state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_end_the_stream_with_a_failed_store_when_the_repository_rejects_the_write() {
        let repository =
            FakeSubmissionsRepository::rejecting(WriteError::conflicting_write("Sample conflict"));
        let super_state = super_state(
            SAMPLE_CIK,
            sec_client(),
            IngestionMode::Incremental,
            repository,
        );

        let expected_result = vec![STORE_SUBMISSIONS.to_string()];

        let result = failed_state_names(super_state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_tag_the_error_with_the_execution_id_of_the_context_when_the_cik_is_not_numeric()
    {
        let super_state = super_state(
            "BRK-B",
            sec_client(),
            IngestionMode::Incremental,
            FakeSubmissionsRepository::new(),
        );

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
    const fn should_implement_auto_traits_for_store_submissions_super_state() {
        implements_auto_traits::<
            ExtractSubmissionsSuperState<
                StoreSubmissions<FakeSubmissionsRepository>,
                FakeSubmissionsRepository,
            >,
        >();
    }
}
