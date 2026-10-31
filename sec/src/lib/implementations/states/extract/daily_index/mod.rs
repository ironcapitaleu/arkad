//! # Detect Filings Super-State
//!
//! Provides the [`DetectFilingsSuperState`], which downloads the EDGAR daily index of one date,
//! writes it to the store, and selects the filings to fetch.
//!
//! The super-state owns the shared SEC client, the `execution_id`, and the storage repository.
//! [`ValidateIndexDate`] and [`DownloadDailyIndex`] hold no repository, so the transitions into
//! [`PrepareSecRequest`] and [`StoreDailyIndex`] take a clone from the super-state.
//!
//! ## Modules
//!
//! - [`builder`]: Builds the [`DetectFilingsSuperState`] at its entry state [`ValidateIndexDate`].
//!
//! ## State Flow
//!
//! [`ValidateIndexDate`] → [`PrepareSecRequest`] of the [`DailyIndex`] kind →
//! [`DownloadDailyIndex`] → [`StoreDailyIndex`] → [`SelectFilings`]. The stream ends after
//! [`SelectFilings`]. [`DetectFilingsSuperState::run_to_filings`] runs the stream and returns the
//! selected [`FilingRef`] values. A day without filings stores no file and returns an empty list.
//!
//! ## Usage
//!
//! ```no_run
//! use sec::implementations::states::extract::daily_index::DetectFilingsSuperState;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use sec::shared::raw_document::RawDocument;
//! use sec::shared::raw_source::RawDailyIndex;
//! use storage::implementations::repository::filesystem::FilesystemRepository;
//! use uuid::Uuid;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let repository = FilesystemRepository::builder()
//!     .writer()
//!     .document_type::<RawDocument<RawDailyIndex>>()
//!     .root("data")
//!     .build();
//! let detect_state = DetectFilingsSuperState::builder()
//!     .date("2026-10-27")
//!     .sec_client(SecClient::default())
//!     .execution_id(Uuid::new_v4())
//!     .repository(repository)
//!     .build();
//! let filings = detect_state
//!     .run_to_filings(|_item| {})
//!     .await
//!     .expect("The live SEC download should succeed");
//! # }
//! ```

pub mod builder;

use std::fmt::{self, Display, Formatter};

use async_trait::async_trait;
use futures_util::StreamExt;
use serde::Serialize;
use state_maschine::prelude::{StateMachine as SMStateMachine, Transition as SMTransition};
use uuid::Uuid;

use crate::error::State as StateError;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::daily_index::builder::{
    DetectFilingsSuperStateBuilder, NoDate, NoExecutionId, NoRepository, NoSecClient,
};
use crate::implementations::states::extract::download_daily_index::DownloadDailyIndex;
use crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
use crate::implementations::states::extract::select_filings::SelectFilings;
use crate::implementations::states::extract::store_daily_index::StoreDailyIndex;
use crate::implementations::states::extract::validate_index_date::ValidateIndexDate;
use crate::prelude::*;
use crate::shared::filing_ref::FilingRef;
use crate::shared::http_client::SecClient as SecClientTrait;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawDailyIndex;
use crate::shared::request::implementations::sec_request::DailyIndex;
use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
use crate::traits::state_machine::stream::{OutputSlot, terminal_stream_with};

/// Input and output data for the [`DetectFilingsSuperState`].
///
/// A placeholder with unit update semantics: the super-state delegates all real data handling to
/// its current inner state, so it carries no data of its own.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct DetectFilingsSuperStateData;

impl SMStateData for DetectFilingsSuperStateData {
    type UpdateType = ();
    fn state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, (): Self::UpdateType) {}
}

impl StateData for DetectFilingsSuperStateData {
    fn update_state(&mut self, (): Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

/// Context for the [`DetectFilingsSuperState`].
///
/// Holds the SEC client of type `C`, the `execution_id`, the repository of type `R`, and the slot
/// for the selected filings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetectFilingsSuperStateContext<R, C> {
    pub(crate) sec_client: C,
    pub(crate) execution_id: Uuid,
    pub(crate) repository: R,
    pub(crate) filings_slot: OutputSlot<Vec<FilingRef>>,
}

impl<R, C> DetectFilingsSuperStateContext<R, C> {
    /// Creates a new [`DetectFilingsSuperStateContext`] from the SEC client, the `execution_id`,
    /// and the repository.
    #[must_use]
    pub fn new(sec_client: C, execution_id: Uuid, repository: R) -> Self {
        Self {
            sec_client,
            execution_id,
            repository,
            filings_slot: OutputSlot::default(),
        }
    }
}

impl<R: RawRepository<RawDailyIndex>, C: SecClientTrait> SMContext
    for DetectFilingsSuperStateContext<R, C>
{
    type UpdateType = ();
    fn context(&self) -> &Self {
        self
    }
    fn update_context(&mut self, (): Self::UpdateType) {}
}

impl<R: RawRepository<RawDailyIndex>, C: SecClientTrait> Context
    for DetectFilingsSuperStateContext<R, C>
{
    fn max_retries(&self) -> u32 {
        0
    }
}

/// Hierarchical super-state that detects the filings of one day.
///
/// Wraps the active state in its `S` type parameter and owns the repository of type `R`, the SEC
/// client of type `C`, and the `execution_id`. It moves through type-safe transitions:
/// [`ValidateIndexDate`] → [`PrepareSecRequest`] → [`DownloadDailyIndex`] → [`StoreDailyIndex`] →
/// [`SelectFilings`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetectFilingsSuperState<S: State, R, C: SecClientTrait = SecClient> {
    current_state: S,
    input: DetectFilingsSuperStateData,
    output: Option<DetectFilingsSuperStateData>,
    context: DetectFilingsSuperStateContext<R, C>,
}

impl<S: State, R, C: SecClientTrait> Display for DetectFilingsSuperState<S, R, C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Detect Filings SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }
}

impl<S: State, R: RawRepository<RawDailyIndex>, C: SecClientTrait> SMState
    for DetectFilingsSuperState<S, R, C>
{
    type InputData = DetectFilingsSuperStateData;
    type OutputData = DetectFilingsSuperStateData;
    type Context = DetectFilingsSuperStateContext<R, C>;

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
impl<S: State, R: RawRepository<RawDailyIndex>, C: SecClientTrait> State
    for DetectFilingsSuperState<S, R, C>
{
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.current_state
            .compute_output_data_async()
            .await
            .map_err(Into::into)
    }
}

impl<S: State, R: RawRepository<RawDailyIndex>, C: SecClientTrait> SMStateMachine<S>
    for DetectFilingsSuperState<S, R, C>
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

impl<S: State, R: RawRepository<RawDailyIndex>, C: SecClientTrait> StateMachine<S>
    for DetectFilingsSuperState<S, R, C>
{
}

impl<S: State, R: RawRepository<RawDailyIndex>, C: SecClientTrait> SMSuperState<S>
    for DetectFilingsSuperState<S, R, C>
{
}

impl<S: State, R: RawRepository<RawDailyIndex>, C: SecClientTrait> SuperState<S>
    for DetectFilingsSuperState<S, R, C>
{
}

impl<R: RawRepository<RawDailyIndex>> DetectFilingsSuperState<ValidateIndexDate, R> {
    /// Returns a [`DetectFilingsSuperStateBuilder`] that holds no field yet.
    #[must_use]
    pub const fn builder()
    -> DetectFilingsSuperStateBuilder<NoDate, NoSecClient, NoExecutionId, NoRepository<R>> {
        DetectFilingsSuperStateBuilder::new()
    }
}

// --- Streaming ---

impl<S: State, R, C: SecClientTrait> DetectFilingsSuperState<S, R, C>
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

    /// Runs the stream to its end and returns the filings that [`SelectFilings`] selected.
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
    /// Panics if the stream ends without an error before [`SelectFilings`] computes its output.
    /// The state flow rules this out.
    pub async fn run_to_filings(
        mut self,
        mut on_item: impl FnMut(&StreamItem),
    ) -> Result<Vec<FilingRef>, StreamError> {
        let filings_slot = OutputSlot::default();
        self.context.filings_slot = filings_slot.clone();
        let mut stream = self.into_run_stream();
        while let Some(item) = stream.next().await {
            on_item(&item?);
        }
        Ok(filings_slot.take().expect(
            "Given a stream that ended without an error, SelectFilings has always put the filings into the slot",
        ))
    }
}

impl<R: RawRepository<RawDailyIndex>, C: SecClientTrait> NonTerminal
    for DetectFilingsSuperState<ValidateIndexDate, R, C>
{
    type Current = ValidateIndexDate;
    type Next = PrepareSecRequest<DailyIndex, R>;
}

// Deviation: the impls that reach `DownloadDailyIndex` pin the response and the error of `C`,
// as the `State` impl of `DownloadDailyIndex` does.
impl<R, C> NonTerminal for DetectFilingsSuperState<PrepareSecRequest<DailyIndex, R>, R, C>
where
    R: RawRepository<RawDailyIndex>,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type Current = PrepareSecRequest<DailyIndex, R>;
    type Next = DownloadDailyIndex<C>;
}

impl<R, C> NonTerminal for DetectFilingsSuperState<DownloadDailyIndex<C>, R, C>
where
    R: RawRepository<RawDailyIndex>,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type Current = DownloadDailyIndex<C>;
    type Next = StoreDailyIndex<R>;
}

impl<R: RawRepository<RawDailyIndex>, C: SecClientTrait> NonTerminal
    for DetectFilingsSuperState<StoreDailyIndex<R>, R, C>
{
    type Current = StoreDailyIndex<R>;
    type Next = SelectFilings;
}

/// Ends the stream after [`SelectFilings`] selects the filings of the day.
impl<R: RawRepository<RawDailyIndex> + 'static, C: SecClientTrait + 'static> IntoStateMachineStream
    for DetectFilingsSuperState<SelectFilings, R, C>
{
    fn into_stream(self, execution_id: Uuid) -> StateMachineStream {
        let filings_slot = self.context.filings_slot.clone();
        terminal_stream_with(self, execution_id, move |state: &SelectFilings| {
            if let Some(output) = state.output_data() {
                filings_slot.put(output.filings.clone());
            }
        })
    }
}

// --- Transitions ---

impl<R: RawRepository<RawDailyIndex>, C: SecClientTrait>
    Transition<ValidateIndexDate, PrepareSecRequest<DailyIndex, R>>
    for DetectFilingsSuperState<ValidateIndexDate, R, C>
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let repository = self.context.repository.clone();
        Ok(DetectFilingsSuperState {
            current_state: PrepareSecRequest::try_from((self.current_state, repository))?,
            input: DetectFilingsSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R: RawRepository<RawDailyIndex>, C: SecClientTrait>
    SMTransition<ValidateIndexDate, PrepareSecRequest<DailyIndex, R>>
    for DetectFilingsSuperState<ValidateIndexDate, R, C>
{
    type NewStateMachine = DetectFilingsSuperState<PrepareSecRequest<DailyIndex, R>, R, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<R, C> Transition<PrepareSecRequest<DailyIndex, R>, DownloadDailyIndex<C>>
    for DetectFilingsSuperState<PrepareSecRequest<DailyIndex, R>, R, C>
where
    R: RawRepository<RawDailyIndex>,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let sec_client = self.context.sec_client.clone();
        Ok(DetectFilingsSuperState {
            current_state: DownloadDailyIndex::try_from((self.current_state, sec_client))?,
            input: DetectFilingsSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R, C> SMTransition<PrepareSecRequest<DailyIndex, R>, DownloadDailyIndex<C>>
    for DetectFilingsSuperState<PrepareSecRequest<DailyIndex, R>, R, C>
where
    R: RawRepository<RawDailyIndex>,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type NewStateMachine = DetectFilingsSuperState<DownloadDailyIndex<C>, R, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<R, C> Transition<DownloadDailyIndex<C>, StoreDailyIndex<R>>
    for DetectFilingsSuperState<DownloadDailyIndex<C>, R, C>
where
    R: RawRepository<RawDailyIndex>,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let repository = self.context.repository.clone();
        Ok(DetectFilingsSuperState {
            current_state: StoreDailyIndex::try_from((
                self.current_state,
                self.context.execution_id,
                repository,
            ))?,
            input: DetectFilingsSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R, C> SMTransition<DownloadDailyIndex<C>, StoreDailyIndex<R>>
    for DetectFilingsSuperState<DownloadDailyIndex<C>, R, C>
where
    R: RawRepository<RawDailyIndex>,
    C: SecClientTrait<Response<ZipBody> = SecResponse<ZipBody>, Error = FailedSecRequest>,
{
    type NewStateMachine = DetectFilingsSuperState<StoreDailyIndex<R>, R, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<R: RawRepository<RawDailyIndex>, C: SecClientTrait>
    Transition<StoreDailyIndex<R>, SelectFilings>
    for DetectFilingsSuperState<StoreDailyIndex<R>, R, C>
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        Ok(DetectFilingsSuperState {
            current_state: SelectFilings::try_from(self.current_state)?,
            input: DetectFilingsSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R: RawRepository<RawDailyIndex>, C: SecClientTrait>
    SMTransition<StoreDailyIndex<R>, SelectFilings>
    for DetectFilingsSuperState<StoreDailyIndex<R>, R, C>
{
    type NewStateMachine = DetectFilingsSuperState<SelectFilings, R, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use storage::WriteError;
    use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::states::extract::download_daily_index::constants::STATE_NAME as DOWNLOAD_DAILY_INDEX;
    use crate::implementations::states::extract::prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
    use crate::implementations::states::extract::select_filings::constants::STATE_NAME as SELECT_FILINGS;
    use crate::implementations::states::extract::store_daily_index::constants::STATE_NAME as STORE_DAILY_INDEX;
    use crate::implementations::states::extract::validate_index_date::constants::STATE_NAME as VALIDATE_INDEX_DATE;
    use crate::shared::cik::Cik;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::raw_document::RawDocument;
    use crate::tests::fixtures::data::daily_index::SAMPLE_DAILY_INDEX;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use crate::tests::fixtures::sample_write_repository::FakeDailyIndexRepository;

    type DailyIndexFilesystemRepository = FilesystemRepository<Write, RawDocument<RawDailyIndex>>;

    const SAMPLE_EXECUTION_ID: Uuid = Uuid::from_u128(7);
    const INDEX_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261027.idx";
    const QTR4_LISTING_URL: &str =
        "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/index.json";
    const QTR4_LISTING: &[u8] = br#"{"directory":{"item":[{"name":"form.20261027.idx"}]}}"#;

    /// Returns a client that answers the index of 2026-10-27 with `index_body`, and lists only
    /// that index for the quarter. Every other index answers 404.
    fn sec_client(index_body: &[u8]) -> RoutingSecClient {
        RoutingSecClient::new()
            .route(INDEX_URL, 200, "application/octet-stream", index_body)
            .route(QTR4_LISTING_URL, 200, "application/json", QTR4_LISTING)
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn filesystem_repository(root: &Path) -> DailyIndexFilesystemRepository {
        FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<RawDailyIndex>>()
            .root(root)
            .build()
    }

    fn super_state<R: RawRepository<RawDailyIndex>>(
        raw_date: &str,
        sec_client: RoutingSecClient,
        repository: R,
    ) -> DetectFilingsSuperState<ValidateIndexDate, R, RoutingSecClient> {
        DetectFilingsSuperState::builder()
            .date(raw_date)
            .sec_client(sec_client)
            .execution_id(SAMPLE_EXECUTION_ID)
            .repository(repository)
            .build()
    }

    async fn filings<R: RawRepository<RawDailyIndex> + 'static>(
        super_state: DetectFilingsSuperState<ValidateIndexDate, R, RoutingSecClient>,
    ) -> Vec<FilingRef> {
        super_state.run_to_filings(|_| {}).await.expect(
            "Given a past date, the fake client, and a writable store, the run should always succeed",
        )
    }

    /// Runs the stream and returns the name of the state that failed.
    async fn failed_state_name(
        super_state: DetectFilingsSuperState<
            ValidateIndexDate,
            FakeDailyIndexRepository,
            RoutingSecClient,
        >,
    ) -> Option<String> {
        super_state
            .run_to_filings(|_| {})
            .await
            .err()
            .map(|error| error.state_name)
    }

    fn filing_ref(cik: &str, accession_number: &str, form: Form) -> FilingRef {
        FilingRef::new(
            Cik::new(cik).expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new(accession_number),
            ParsedForm::new(form, false),
            NaiveDate::from_ymd_opt(2026, 10, 27)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    #[test]
    #[expect(
        clippy::redundant_clone,
        reason = "False positive: `state_name` returns `impl ToString`, which is a `String` here. Clippy flags `.to_string()` as a clone, but removing it does not compile"
    )]
    fn should_return_super_state_name_with_current_state_when_in_validate_index_date_state() {
        let super_state = super_state(
            "2026-10-27",
            RoutingSecClient::new(),
            FakeDailyIndexRepository::new(),
        );

        let expected_result = "Detect Filings SuperState (Current: Validate Index Date)";

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_index_under_sec_daily_index_when_the_day_has_filings() {
        let root = temporary_root();
        let super_state = super_state(
            "2026-10-27",
            sec_client(SAMPLE_DAILY_INDEX.as_bytes()),
            filesystem_repository(root.path()),
        );

        let expected_result = SAMPLE_DAILY_INDEX.as_bytes().to_vec();

        filings(super_state).await;
        let result = std::fs::read(root.path().join("sec/daily-index/form.20261027.idx")).expect(
            "Given a run that completed without an error, the stored file should always exist",
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_selected_filings_when_the_day_has_filings() {
        let root = temporary_root();
        let super_state = super_state(
            "2026-10-27",
            sec_client(SAMPLE_DAILY_INDEX.as_bytes()),
            filesystem_repository(root.path()),
        );

        let expected_result = vec![
            filing_ref("1702744", "0001702744-26-000031", Form::TenK),
            filing_ref("1053507", "0001053507-26-000157", Form::TenQ),
            filing_ref("1047127", "0001047127-26-000052", Form::TenQ),
            filing_ref("109563", "0000109563-26-000054", Form::TenQ),
            filing_ref("811156", "0000811156-26-000032", Form::TenQ),
            filing_ref("2113605", "0001213900-26-112626", Form::TwentyF),
        ];

        let result = filings(super_state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_the_store_empty_when_the_day_has_no_filings() {
        let root = temporary_root();
        let super_state = super_state(
            "2026-10-25",
            sec_client(SAMPLE_DAILY_INDEX.as_bytes()),
            filesystem_repository(root.path()),
        );

        let expected_result = true;

        filings(super_state).await;
        let result = std::fs::read_dir(root.path())
            .expect("Given a temp directory that exists for the whole test, reading it should always succeed")
            .next()
            .is_none();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_an_empty_list_when_the_day_has_no_filings() {
        let root = temporary_root();
        let super_state = super_state(
            "2026-10-25",
            sec_client(SAMPLE_DAILY_INDEX.as_bytes()),
            filesystem_repository(root.path()),
        );

        let expected_result: Vec<FilingRef> = Vec::new();

        let result = filings(super_state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_hand_every_stream_item_to_the_caller_in_state_order_when_running_to_the_filings()
     {
        let super_state = super_state(
            "2026-10-27",
            sec_client(SAMPLE_DAILY_INDEX.as_bytes()),
            FakeDailyIndexRepository::new(),
        );

        let expected_result = vec![
            (StreamEvent::StateStarted, VALIDATE_INDEX_DATE),
            (StreamEvent::StateCompleted, VALIDATE_INDEX_DATE),
            (StreamEvent::TransitionCompleted, VALIDATE_INDEX_DATE),
            (StreamEvent::StateStarted, PREPARE_SEC_REQUEST),
            (StreamEvent::StateCompleted, PREPARE_SEC_REQUEST),
            (StreamEvent::TransitionCompleted, PREPARE_SEC_REQUEST),
            (StreamEvent::StateStarted, DOWNLOAD_DAILY_INDEX),
            (StreamEvent::StateCompleted, DOWNLOAD_DAILY_INDEX),
            (StreamEvent::TransitionCompleted, DOWNLOAD_DAILY_INDEX),
            (StreamEvent::StateStarted, STORE_DAILY_INDEX),
            (StreamEvent::StateCompleted, STORE_DAILY_INDEX),
            (StreamEvent::TransitionCompleted, STORE_DAILY_INDEX),
            (StreamEvent::StateStarted, SELECT_FILINGS),
            (StreamEvent::StateCompleted, SELECT_FILINGS),
        ];

        let mut items = Vec::new();
        super_state
            .run_to_filings(|item| items.push((item.event.clone(), item.state_name.clone())))
            .await
            .expect("Given a past date, the fake client, and a fake store, the run should always succeed");
        let result: Vec<(StreamEvent, &str)> = items
            .iter()
            .map(|(event, state_name)| (event.clone(), state_name.as_str()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_in_validate_index_date_when_the_date_is_invalid() {
        let super_state = super_state(
            "2026-13-01",
            sec_client(SAMPLE_DAILY_INDEX.as_bytes()),
            FakeDailyIndexRepository::new(),
        );

        let expected_result = Some(VALIDATE_INDEX_DATE.to_string());

        let result = failed_state_name(super_state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_in_download_daily_index_when_the_index_and_the_listing_are_missing() {
        let super_state = super_state(
            "2026-10-27",
            RoutingSecClient::new(),
            FakeDailyIndexRepository::new(),
        );

        let expected_result = Some(DOWNLOAD_DAILY_INDEX.to_string());

        let result = failed_state_name(super_state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_in_select_filings_when_the_index_is_malformed() {
        let super_state = super_state(
            "2026-10-27",
            sec_client(b"An index without a header line\n"),
            FakeDailyIndexRepository::new(),
        );

        let expected_result = Some(SELECT_FILINGS.to_string());

        let result = failed_state_name(super_state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_in_store_daily_index_when_the_repository_rejects_the_write() {
        let repository =
            FakeDailyIndexRepository::rejecting(WriteError::conflicting_write("Sample conflict"));
        let super_state = super_state(
            "2026-10-27",
            sec_client(SAMPLE_DAILY_INDEX.as_bytes()),
            repository,
        );

        let expected_result = Some(STORE_DAILY_INDEX.to_string());

        let result = failed_state_name(super_state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_tag_stream_with_execution_id_of_context_when_streaming_the_run() {
        let super_state = super_state(
            "2026-13-01",
            RoutingSecClient::new(),
            FakeDailyIndexRepository::new(),
        );

        let expected_result = vec![SAMPLE_EXECUTION_ID];

        let result: Vec<Uuid> = super_state
            .into_run_stream()
            .filter_map(|item| async move { item.err().map(|error| error.execution_id) })
            .collect()
            .await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_filings_when_a_clone_empties_the_slot_it_was_built_with() {
        let super_state = super_state(
            "2026-10-27",
            sec_client(SAMPLE_DAILY_INDEX.as_bytes()),
            FakeDailyIndexRepository::new(),
        );
        let built_slot = super_state.context.filings_slot.clone();

        let expected_result = Some(6);

        let result = super_state
            .run_to_filings(|_| {
                built_slot.take();
            })
            .await
            .ok()
            .map(|filings| filings.len());

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_select_filings_super_state() {
        implements_auto_traits::<DetectFilingsSuperState<SelectFilings, FakeDailyIndexRepository>>(
        );
    }
}
