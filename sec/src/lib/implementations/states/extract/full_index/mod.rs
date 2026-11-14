//! # Detect Quarter Filings Super-State
//!
//! Provides the [`DetectQuarterFilingsSuperState`], which downloads the EDGAR full index of one
//! quarter, writes it to the store, and selects the filings of a range of dates filed.
//!
//! The super-state owns the shared SEC client, the `execution_id`, the ingestion mode of the run,
//! the storage repository, and the range of dates filed. The catch-up of `backfill` runs one
//! super-state for each quarter that its date range touches.
//!
//! ## State Flow
//!
//! [`PrepareSecRequest`] of the [`FullIndex`] kind → [`ExecuteSecRequest`] of the [`FullIndex`]
//! kind → [`StoreFullIndex`] → [`SelectFullIndexFilings`]. The stream ends after
//! [`SelectFullIndexFilings`]. [`DetectQuarterFilingsSuperState::run_to_detected_filings`] runs the
//! stream and returns the selected filings with the stored full index they came from.
//!
//! ## Usage
//!
//! ```no_run
//! use chrono::NaiveDate;
//! use sec::implementations::states::extract::full_index::{
//!     DetectQuarterFilingsSuperState, DetectQuarterFilingsSuperStateContext,
//! };
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use sec::shared::raw_document::RawDocument;
//! use sec::shared::raw_source::RawFullIndex;
//! use storage::implementations::repository::filesystem::{FilesystemRepository, IngestionMode};
//! use uuid::Uuid;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let first_filed_date = NaiveDate::from_ymd_opt(2026, 10, 1).expect("A hardcoded date should build");
//! let last_filed_date = NaiveDate::from_ymd_opt(2026, 10, 31).expect("A hardcoded date should build");
//! let repository = FilesystemRepository::builder()
//!     .writer()
//!     .document_type::<RawDocument<RawFullIndex>>()
//!     .root("data")
//!     .build();
//! let context = DetectQuarterFilingsSuperStateContext::new(
//!     SecClient::default(),
//!     Uuid::new_v4(),
//!     IngestionMode::Incremental,
//!     repository,
//!     first_filed_date,
//!     last_filed_date,
//! );
//! let detected = DetectQuarterFilingsSuperState::new(context)
//!     .run_to_detected_filings(|_item| {})
//!     .await
//!     .expect("The live SEC download should succeed");
//! # }
//! ```

use std::fmt::{self, Display, Formatter};

use async_trait::async_trait;
use chrono::NaiveDate;
use futures_util::StreamExt;
use serde::Serialize;
use state_maschine::prelude::{StateMachine as SMStateMachine, Transition as SMTransition};
use storage::implementations::repository::filesystem::IngestionMode;
use uuid::Uuid;

use crate::error::State as StateError;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::daily_index::DetectedFilings;
use crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest;
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use crate::implementations::states::extract::select_full_index_filings::SelectFullIndexFilings;
use crate::implementations::states::extract::store_full_index::StoreFullIndex;
use crate::prelude::*;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::quarter::Quarter;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawFullIndex;
use crate::shared::request::implementations::sec_request::FullIndex;
use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
use crate::traits::state_machine::stream::{OutputSlot, terminal_stream_with};

/// Input and output data for the [`DetectQuarterFilingsSuperState`].
///
/// A placeholder with unit updates. The current inner state holds all the data, so the
/// super-state holds none.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct DetectQuarterFilingsSuperStateData;

impl SMStateData for DetectQuarterFilingsSuperStateData {
    type UpdateType = ();
    fn state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, (): Self::UpdateType) {}
}

impl StateData for DetectQuarterFilingsSuperStateData {
    fn update_state(&mut self, (): Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

/// Context for the [`DetectQuarterFilingsSuperState`].
///
/// Holds the SEC client of type `C`, the `execution_id`, the ingestion mode, the repository of type
/// `R`, the range of dates filed to keep, and the slot for the selected filings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetectQuarterFilingsSuperStateContext<R, C> {
    pub(crate) sec_client: C,
    pub(crate) execution_id: Uuid,
    pub(crate) ingestion_mode: IngestionMode,
    pub(crate) repository: R,
    pub(crate) first_filed_date: NaiveDate,
    pub(crate) last_filed_date: NaiveDate,
    pub(crate) filings_slot: OutputSlot<DetectedFilings>,
}

impl<R, C> DetectQuarterFilingsSuperStateContext<R, C> {
    /// Creates a new [`DetectQuarterFilingsSuperStateContext`] from the SEC client, the
    /// `execution_id`, the ingestion mode, the repository, and the first and the last date filed
    /// to keep.
    #[must_use]
    pub fn new(
        sec_client: C,
        execution_id: Uuid,
        ingestion_mode: IngestionMode,
        repository: R,
        first_filed_date: NaiveDate,
        last_filed_date: NaiveDate,
    ) -> Self {
        Self {
            sec_client,
            execution_id,
            ingestion_mode,
            repository,
            first_filed_date,
            last_filed_date,
            filings_slot: OutputSlot::default(),
        }
    }
}

impl<R: RawRepository<RawFullIndex>, C: SecClientTrait> SMContext
    for DetectQuarterFilingsSuperStateContext<R, C>
{
    type UpdateType = ();
    fn context(&self) -> &Self {
        self
    }
    fn update_context(&mut self, (): Self::UpdateType) {}
}

impl<R: RawRepository<RawFullIndex>, C: SecClientTrait> Context
    for DetectQuarterFilingsSuperStateContext<R, C>
{
    fn max_retries(&self) -> u32 {
        0
    }
}

/// Hierarchical super-state that detects the filings of a range of dates filed in one quarter.
///
/// Wraps the active state in its `S` type parameter and owns the repository of type `R`, the SEC
/// client of type `C`, and the `execution_id`. It moves through type-safe transitions:
/// [`PrepareSecRequest`] → [`ExecuteSecRequest`] → [`StoreFullIndex`] → [`SelectFullIndexFilings`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetectQuarterFilingsSuperState<S: State, R, C: SecClientTrait = SecClient> {
    current_state: S,
    input: DetectQuarterFilingsSuperStateData,
    output: Option<DetectQuarterFilingsSuperStateData>,
    context: DetectQuarterFilingsSuperStateContext<R, C>,
}

impl<R: RawRepository<RawFullIndex>, C: SecClientTrait>
    DetectQuarterFilingsSuperState<PrepareSecRequest<FullIndex>, R, C>
{
    /// Creates a new [`DetectQuarterFilingsSuperState`] at its entry state, the
    /// [`PrepareSecRequest`] of the full index of the quarter of the first date filed of
    /// `context`.
    ///
    /// The quarter comes from the context, so the downloaded file always holds the first date
    /// filed. A last date filed in a later quarter fails the run in [`SelectFullIndexFilings`].
    #[must_use]
    pub fn new(context: DetectQuarterFilingsSuperStateContext<R, C>) -> Self {
        let quarter = Quarter::of(context.first_filed_date);
        Self {
            current_state: PrepareSecRequest::new(
                PrepareSecRequestInput::new(quarter),
                PrepareSecRequestContext::new(quarter),
            ),
            input: DetectQuarterFilingsSuperStateData,
            output: None,
            context,
        }
    }
}

impl<S: State, R: RawRepository<RawFullIndex>, C: SecClientTrait> Display
    for DetectQuarterFilingsSuperState<S, R, C>
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Detect Quarter Filings SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }
}

impl<S: State, R: RawRepository<RawFullIndex>, C: SecClientTrait> SMState
    for DetectQuarterFilingsSuperState<S, R, C>
{
    type InputData = DetectQuarterFilingsSuperStateData;
    type OutputData = DetectQuarterFilingsSuperStateData;
    type Context = DetectQuarterFilingsSuperStateContext<R, C>;

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
impl<S: State, R: RawRepository<RawFullIndex>, C: SecClientTrait> State
    for DetectQuarterFilingsSuperState<S, R, C>
{
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.current_state
            .compute_output_data_async()
            .await
            .map_err(Into::into)
    }
}

impl<S: State, R: RawRepository<RawFullIndex>, C: SecClientTrait> SMStateMachine<S>
    for DetectQuarterFilingsSuperState<S, R, C>
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

impl<S: State, R: RawRepository<RawFullIndex>, C: SecClientTrait> StateMachine<S>
    for DetectQuarterFilingsSuperState<S, R, C>
{
}

impl<S: State, R: RawRepository<RawFullIndex>, C: SecClientTrait> SMSuperState<S>
    for DetectQuarterFilingsSuperState<S, R, C>
{
}

impl<S: State, R: RawRepository<RawFullIndex>, C: SecClientTrait> SuperState<S>
    for DetectQuarterFilingsSuperState<S, R, C>
{
}

// --- Streaming ---

impl<S: State, R, C: SecClientTrait> DetectQuarterFilingsSuperState<S, R, C>
where
    Self: IntoStateMachineStream,
{
    /// Returns the stream, tagged with the `execution_id` that the context holds.
    ///
    /// Use it instead of `into_stream`, so the stream and the stored document name the same run.
    pub fn into_run_stream(self) -> StateMachineStream {
        let execution_id = self.context.execution_id;
        self.into_stream(execution_id)
    }

    /// Runs the stream to its end and returns the selected filings with the stored full index
    /// they came from.
    ///
    /// Calls `on_item` with each successful stream item, in stream order. Each call gets its own
    /// slot for the filings, so clones of one super-state never share it.
    ///
    /// # Errors
    ///
    /// Returns the [`StreamError`] of the first state that fails. The stream ends at that error.
    ///
    /// # Panics
    ///
    /// Panics if the stream ends without an error before [`SelectFullIndexFilings`] computes its
    /// output. The state flow rules this out.
    pub async fn run_to_detected_filings(
        mut self,
        mut on_item: impl FnMut(&StreamItem),
    ) -> Result<DetectedFilings, StreamError> {
        let filings_slot = OutputSlot::default();
        self.context.filings_slot = filings_slot.clone();
        let mut stream = self.into_run_stream();
        while let Some(item) = stream.next().await {
            on_item(&item?);
        }
        Ok(filings_slot.take().expect(
            "Given a stream that ended without an error, SelectFullIndexFilings has always put the filings into the slot",
        ))
    }
}

// Deviation: the impls that reach `ExecuteSecRequest` pin the response and the error of `C`.
// See the comment on `State for ExecuteSecRequest`.
impl<R, C> NonTerminal for DetectQuarterFilingsSuperState<PrepareSecRequest<FullIndex>, R, C>
where
    R: RawRepository<RawFullIndex>,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type Current = PrepareSecRequest<FullIndex>;
    type Next = ExecuteSecRequest<FullIndex, C>;
}

impl<R, C> NonTerminal for DetectQuarterFilingsSuperState<ExecuteSecRequest<FullIndex, C>, R, C>
where
    R: RawRepository<RawFullIndex>,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type Current = ExecuteSecRequest<FullIndex, C>;
    type Next = StoreFullIndex<R>;
}

impl<R: RawRepository<RawFullIndex>, C: SecClientTrait> NonTerminal
    for DetectQuarterFilingsSuperState<StoreFullIndex<R>, R, C>
{
    type Current = StoreFullIndex<R>;
    type Next = SelectFullIndexFilings;
}

/// Ends the stream after [`SelectFullIndexFilings`] selects the filings of the range.
impl<R: RawRepository<RawFullIndex> + 'static, C: SecClientTrait + 'static> IntoStateMachineStream
    for DetectQuarterFilingsSuperState<SelectFullIndexFilings, R, C>
{
    fn into_stream(self, execution_id: Uuid) -> StateMachineStream {
        let filings_slot = self.context.filings_slot.clone();
        terminal_stream_with(self, execution_id, move |state: &SelectFullIndexFilings| {
            if let Some(output) = state.output_data() {
                filings_slot.put(DetectedFilings {
                    filings: output.filings.clone(),
                    index: Some(state.input_data().index.input_document()),
                });
            }
        })
    }
}

// --- Transitions ---

impl<R, C> Transition<PrepareSecRequest<FullIndex>, ExecuteSecRequest<FullIndex, C>>
    for DetectQuarterFilingsSuperState<PrepareSecRequest<FullIndex>, R, C>
where
    R: RawRepository<RawFullIndex>,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let sec_client = self.context.sec_client.clone();
        Ok(DetectQuarterFilingsSuperState {
            current_state: ExecuteSecRequest::try_from((self.current_state, sec_client))?,
            input: DetectQuarterFilingsSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R, C> SMTransition<PrepareSecRequest<FullIndex>, ExecuteSecRequest<FullIndex, C>>
    for DetectQuarterFilingsSuperState<PrepareSecRequest<FullIndex>, R, C>
where
    R: RawRepository<RawFullIndex>,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type NewStateMachine = DetectQuarterFilingsSuperState<ExecuteSecRequest<FullIndex, C>, R, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<R, C> Transition<ExecuteSecRequest<FullIndex, C>, StoreFullIndex<R>>
    for DetectQuarterFilingsSuperState<ExecuteSecRequest<FullIndex, C>, R, C>
where
    R: RawRepository<RawFullIndex>,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        Ok(DetectQuarterFilingsSuperState {
            current_state: StoreFullIndex::try_from((
                self.current_state,
                self.context.execution_id,
                self.context.ingestion_mode,
                self.context.repository.clone(),
            ))?,
            input: DetectQuarterFilingsSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R, C> SMTransition<ExecuteSecRequest<FullIndex, C>, StoreFullIndex<R>>
    for DetectQuarterFilingsSuperState<ExecuteSecRequest<FullIndex, C>, R, C>
where
    R: RawRepository<RawFullIndex>,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type NewStateMachine = DetectQuarterFilingsSuperState<StoreFullIndex<R>, R, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<R: RawRepository<RawFullIndex>, C: SecClientTrait>
    Transition<StoreFullIndex<R>, SelectFullIndexFilings>
    for DetectQuarterFilingsSuperState<StoreFullIndex<R>, R, C>
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        Ok(DetectQuarterFilingsSuperState {
            current_state: SelectFullIndexFilings::try_from((
                self.current_state,
                self.context.first_filed_date,
                self.context.last_filed_date,
            ))?,
            input: DetectQuarterFilingsSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R: RawRepository<RawFullIndex>, C: SecClientTrait>
    SMTransition<StoreFullIndex<R>, SelectFullIndexFilings>
    for DetectQuarterFilingsSuperState<StoreFullIndex<R>, R, C>
{
    type NewStateMachine = DetectQuarterFilingsSuperState<SelectFullIndexFilings, R, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use storage::Sha256Digest;
    use storage::implementations::repository::filesystem::{
        Document, DocumentKey, FilesystemRepository, InputDocument, Write,
    };
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::states::extract::execute_sec_request::constants::STATE_NAME as EXECUTE_SEC_REQUEST;
    use crate::implementations::states::extract::select_full_index_filings::constants::STATE_NAME as SELECT_FULL_INDEX_FILINGS;
    use crate::shared::raw_document::RawDocument;
    use crate::tests::fixtures::data::full_index::SAMPLE_FULL_INDEX_2026_Q4;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use crate::tests::fixtures::sample_write_repository::FakeFullIndexRepository;

    type FullIndexFilesystemRepository = FilesystemRepository<Write, RawDocument<RawFullIndex>>;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);
    const QTR4_URL: &str = "https://www.sec.gov/Archives/edgar/full-index/2026/QTR4/form.idx";
    const QTR4_KEY: &str = "sec/full-index/2026/QTR4/form.idx";

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, the date should always build")
    }

    /// Returns a client that answers the full index of 2026 Q4 with `status` and `body`.
    fn sec_client(status: u16, body: &[u8]) -> RoutingSecClient {
        RoutingSecClient::new().route(QTR4_URL, status, "application/octet-stream", body)
    }

    /// Returns the super-state that keeps the dates filed from 2026-10-01 to 2026-10-01, with the
    /// ingestion mode `incremental`.
    fn super_state<R: RawRepository<RawFullIndex>>(
        sec_client: RoutingSecClient,
        repository: R,
    ) -> DetectQuarterFilingsSuperState<PrepareSecRequest<FullIndex>, R, RoutingSecClient> {
        super_state_for(
            sec_client,
            repository,
            IngestionMode::Incremental,
            date(2026, 10, 1),
            date(2026, 10, 1),
        )
    }

    /// Returns the super-state that keeps the dates filed from `first` to `last`, with
    /// `ingestion_mode`.
    fn super_state_for<R: RawRepository<RawFullIndex>>(
        sec_client: RoutingSecClient,
        repository: R,
        ingestion_mode: IngestionMode,
        first: NaiveDate,
        last: NaiveDate,
    ) -> DetectQuarterFilingsSuperState<PrepareSecRequest<FullIndex>, R, RoutingSecClient> {
        let context = DetectQuarterFilingsSuperStateContext::new(
            sec_client,
            SAMPLE_EXECUTION_ID,
            ingestion_mode,
            repository,
            first,
            last,
        );
        DetectQuarterFilingsSuperState::new(context)
    }

    async fn detected<R: RawRepository<RawFullIndex> + 'static>(
        super_state: DetectQuarterFilingsSuperState<
            PrepareSecRequest<FullIndex>,
            R,
            RoutingSecClient,
        >,
    ) -> DetectedFilings {
        super_state
            .run_to_detected_filings(|_| {})
            .await
            .expect("Given the fake client and a writable store, the run should always succeed")
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    #[test]
    #[expect(
        clippy::redundant_clone,
        reason = "False positive: `state_name` returns `impl ToString`, which is a `String` here. Clippy flags `.to_string()` as a clone, but removing it does not compile"
    )]
    fn should_return_super_state_name_with_current_state_when_in_the_entry_state() {
        let super_state = super_state(RoutingSecClient::new(), FakeFullIndexRepository::new());

        let expected_result = "Detect Quarter Filings SuperState (Current: Prepare SEC Request)";

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_selected_filings_of_the_range_for_sample_full_index() {
        let super_state = super_state(
            sec_client(200, SAMPLE_FULL_INDEX_2026_Q4.as_bytes()),
            FakeFullIndexRepository::new(),
        );

        let expected_result = vec![
            "0001437749-26-031727".to_string(),
            "0000063754-26-000329".to_string(),
            "0001493152-26-045181".to_string(),
        ];

        let result: Vec<String> = detected(super_state)
            .await
            .filings
            .iter()
            .map(ToString::to_string)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_filings_of_the_last_date_filed_when_the_range_spans_two_days() {
        let super_state = super_state_for(
            sec_client(200, SAMPLE_FULL_INDEX_2026_Q4.as_bytes()),
            FakeFullIndexRepository::new(),
            IngestionMode::Incremental,
            date(2026, 10, 1),
            date(2026, 10, 2),
        );

        let expected_result = vec![
            "0001437749-26-031727".to_string(),
            "0000940944-26-000042".to_string(),
            "0000063754-26-000329".to_string(),
            "0001493152-26-045181".to_string(),
        ];

        let result: Vec<String> = detected(super_state)
            .await
            .filings
            .iter()
            .map(ToString::to_string)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_in_select_full_index_filings_when_the_last_date_filed_is_in_the_next_quarter()
     {
        let super_state = super_state_for(
            sec_client(200, SAMPLE_FULL_INDEX_2026_Q4.as_bytes()),
            FakeFullIndexRepository::new(),
            IngestionMode::Incremental,
            date(2026, 12, 31),
            date(2027, 1, 1),
        );

        let expected_result = Some(SELECT_FULL_INDEX_FILINGS.to_string());

        let result = super_state
            .run_to_detected_filings(|_| {})
            .await
            .err()
            .map(|error| error.state_name);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_stored_full_index_as_the_input_document_of_the_filings() {
        let super_state = super_state(
            sec_client(200, SAMPLE_FULL_INDEX_2026_Q4.as_bytes()),
            FakeFullIndexRepository::new(),
        );

        let expected_result = Some(InputDocument::new(
            DocumentKey::new(QTR4_KEY)
                .expect("Given a hardcoded valid relative path, the key should always build"),
            Sha256Digest::of(SAMPLE_FULL_INDEX_2026_Q4.as_bytes()),
        ));

        let result = detected(super_state).await.index;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_full_index_under_its_key_when_the_run_succeeds() {
        let root = temporary_root();
        let repository: FullIndexFilesystemRepository = FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<RawFullIndex>>()
            .root(root.path())
            .build();
        let super_state = super_state(
            sec_client(200, SAMPLE_FULL_INDEX_2026_Q4.as_bytes()),
            repository,
        );

        let expected_result = SAMPLE_FULL_INDEX_2026_Q4.as_bytes().to_vec();

        detected(super_state).await;
        let result = std::fs::read(root.path().join(QTR4_KEY)).expect(
            "Given a run that completed without an error, the stored file should always exist",
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_the_ingestion_mode_of_the_context_when_storing_the_full_index() {
        let repository = FakeFullIndexRepository::new();
        let super_state = super_state_for(
            sec_client(200, SAMPLE_FULL_INDEX_2026_Q4.as_bytes()),
            repository.clone(),
            IngestionMode::Backfill,
            date(2026, 10, 1),
            date(2026, 10, 1),
        );

        let expected_result = vec![IngestionMode::Backfill];

        detected(super_state).await;
        let result: Vec<IngestionMode> = repository
            .persisted()
            .iter()
            .map(|document| document.metadata().ingestion_mode())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_in_execute_sec_request_when_the_sec_answers_404() {
        let super_state = super_state(sec_client(404, b""), FakeFullIndexRepository::new());

        let expected_result = Some(EXECUTE_SEC_REQUEST.to_string());

        let result = super_state
            .run_to_detected_filings(|_| {})
            .await
            .err()
            .map(|error| error.state_name);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_in_select_full_index_filings_when_the_index_has_no_header() {
        let super_state = super_state(
            sec_client(200, b"no header line"),
            FakeFullIndexRepository::new(),
        );

        let expected_result = Some(SELECT_FULL_INDEX_FILINGS.to_string());

        let result = super_state
            .run_to_detected_filings(|_| {})
            .await
            .err()
            .map(|error| error.state_name);

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_detect_quarter_filings_super_state() {
        implements_value_traits::<
            DetectQuarterFilingsSuperState<
                PrepareSecRequest<FullIndex>,
                FakeFullIndexRepository,
                RoutingSecClient,
            >,
        >();
    }

    const fn implements_stream<T: IntoStateMachineStream>() {}
    #[test]
    const fn should_implement_into_state_machine_stream_for_the_entry_state() {
        implements_stream::<
            DetectQuarterFilingsSuperState<
                PrepareSecRequest<FullIndex>,
                FullIndexFilesystemRepository,
                SecClient,
            >,
        >();
    }
}
