//! # Extract Filing Super-State
//!
//! Provides the [`ExtractFilingSuperState`], which downloads the XBRL instance and the label file
//! of one filing and writes them to the store.
//!
//! The super-state owns the [`FilingRef`], the shared SEC client, the `execution_id`, the ingestion
//! mode of the run, and the two storage repositories. The caller sets the mode, and the metadata of
//! the stored documents records it.
//!
//! ## Modules
//!
//! - [`builder`]: Builds the [`ExtractFilingSuperState`] at its entry state, the
//!   [`PrepareSecRequest`] of the [`FilingIndex`] kind.
//!
//! ## State Flow
//!
//! [`PrepareSecRequest`] of the [`FilingIndex`] kind → [`ExecuteSecRequest`] of the
//! [`FilingIndex`] kind → [`SelectFilingDocuments`] → [`PrepareSecRequest`] of the
//! [`FilingDocumentSet`] kind → [`ExecuteSecRequest`] of the [`FilingDocumentSet`] kind →
//! [`StoreFilingDocuments`]. The stream ends after [`StoreFilingDocuments`].
//! [`ExtractFilingSuperState::run_to_documents`] runs the stream and returns the
//! [`FilingDocumentsRef`] of the stored files.
//!
//! A failed request for the instance or the label file fails the filing. The super-state stores
//! no file of a filing whose documents request failed.
//!
//! ## Usage
//!
//! ```no_run
//! use chrono::NaiveDate;
//! use sec::implementations::states::extract::filing::ExtractFilingSuperState;
//! use sec::shared::cik::Cik;
//! use sec::shared::filing_ref::FilingRef;
//! use sec::shared::financial::accession_number::AccessionNumber;
//! use sec::shared::financial::form::{Form, ParsedForm};
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use sec::shared::raw_document::RawDocument;
//! use sec::shared::raw_source::{RawXbrlInstance, RawXbrlLabels};
//! use storage::implementations::repository::filesystem::{FilesystemRepository, IngestionMode};
//! use uuid::Uuid;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let filing_ref = FilingRef::new(
//!     Cik::new("3545").expect("The CIK is valid"),
//!     AccessionNumber::new("0000003545-26-000047").expect("The accession number is valid"),
//!     ParsedForm::new(Form::TenQ, false),
//!     NaiveDate::from_ymd_opt(2026, 9, 9).expect("The date exists"),
//! );
//! let instance_repository = FilesystemRepository::builder()
//!     .writer()
//!     .document_type::<RawDocument<RawXbrlInstance>>()
//!     .root("data")
//!     .build();
//! let labels_repository = FilesystemRepository::builder()
//!     .writer()
//!     .document_type::<RawDocument<RawXbrlLabels>>()
//!     .root("data")
//!     .build();
//! let extract_state = ExtractFilingSuperState::builder()
//!     .filing_ref(filing_ref)
//!     .sec_client(SecClient::default())
//!     .execution_id(Uuid::new_v4())
//!     .ingestion_mode(IngestionMode::Incremental)
//!     .instance_repository(instance_repository)
//!     .labels_repository(labels_repository)
//!     .build();
//! let documents = extract_state
//!     .run_to_documents(|_item| {})
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
use storage::implementations::repository::filesystem::{IngestionMode, InputDocument};
use uuid::Uuid;

use crate::error::State as StateError;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest;
use crate::implementations::states::extract::filing::builder::{
    ExtractFilingSuperStateBuilder, NoExecutionId, NoFilingRef, NoIngestionMode,
    NoInstanceRepository, NoLabelsRepository, NoSecClient,
};
use crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
use crate::implementations::states::extract::select_filing_documents::SelectFilingDocuments;
use crate::implementations::states::extract::store_filing_documents::StoreFilingDocuments;
use crate::prelude::*;
use crate::shared::filing_documents_ref::FilingDocumentsRef;
use crate::shared::filing_ref::FilingRef;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::http_client::{Executes, SecClient as SecClientTrait};
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::{RawXbrlInstance, RawXbrlLabels};
use crate::shared::request::implementations::sec_request::{FilingDocumentSet, FilingIndex};
use crate::traits::state_machine::stream::{OutputSlot, terminal_stream_with};

/// Input and output data for the [`ExtractFilingSuperState`].
///
/// A placeholder with unit update semantics: the super-state delegates all real data handling to
/// its current inner state, so it carries no data of its own.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ExtractFilingSuperStateData;

impl SMStateData for ExtractFilingSuperStateData {
    type UpdateType = ();
    fn state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, (): Self::UpdateType) {}
}

impl StateData for ExtractFilingSuperStateData {
    fn update_state(&mut self, (): Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

/// Context for the [`ExtractFilingSuperState`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractFilingSuperStateContext<I, L, C> {
    pub(crate) filing_ref: FilingRef,
    pub(crate) sec_client: C,
    pub(crate) execution_id: Uuid,
    pub(crate) ingestion_mode: IngestionMode,
    pub(crate) instance_repository: I,
    pub(crate) labels_repository: L,
    pub(crate) documents_slot: OutputSlot<FilingDocumentsRef>,
    pub(crate) inputs: Vec<InputDocument>,
}

impl<I, L, C> ExtractFilingSuperStateContext<I, L, C> {
    /// Creates a new [`ExtractFilingSuperStateContext`] from the [`FilingRef`], the SEC client,
    /// the `execution_id`, the ingestion mode, and the two storage repositories.
    #[must_use]
    pub fn new(
        filing_ref: FilingRef,
        sec_client: C,
        execution_id: Uuid,
        ingestion_mode: IngestionMode,
        instance_repository: I,
        labels_repository: L,
    ) -> Self {
        Self {
            filing_ref,
            sec_client,
            execution_id,
            ingestion_mode,
            instance_repository,
            labels_repository,
            documents_slot: OutputSlot::default(),
            inputs: Vec::new(),
        }
    }

    /// Returns this context with the documents that the filing came from.
    #[must_use]
    pub fn with_inputs(self, inputs: Vec<InputDocument>) -> Self {
        Self { inputs, ..self }
    }
}

impl<I, L, C> SMContext for ExtractFilingSuperStateContext<I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    type UpdateType = ();
    fn context(&self) -> &Self {
        self
    }
    fn update_context(&mut self, (): Self::UpdateType) {}
}

impl<I, L, C> Context for ExtractFilingSuperStateContext<I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    fn max_retries(&self) -> u32 {
        0
    }
}

/// Hierarchical super-state that extracts the XBRL instance and the label file of one filing.
///
/// Wraps the active state in its `S` type parameter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractFilingSuperState<
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait = SecClient,
> {
    current_state: S,
    input: ExtractFilingSuperStateData,
    output: Option<ExtractFilingSuperStateData>,
    context: ExtractFilingSuperStateContext<I, L, C>,
}

impl<S, I, L, C> Display for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Extract Filing SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }
}

impl<S, I, L, C> SMState for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    type InputData = ExtractFilingSuperStateData;
    type OutputData = ExtractFilingSuperStateData;
    type Context = ExtractFilingSuperStateContext<I, L, C>;

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
impl<S, I, L, C> State for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.current_state
            .compute_output_data_async()
            .await
            .map_err(Into::into)
    }
}

impl<S, I, L, C> SMStateMachine<S> for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
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

impl<S, I, L, C> StateMachine<S> for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
}

impl<S, I, L, C> SMSuperState<S> for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
}

impl<S, I, L, C> SuperState<S> for ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
}

impl<I, L> ExtractFilingSuperState<PrepareSecRequest<FilingIndex>, I, L>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
{
    /// Returns an [`ExtractFilingSuperStateBuilder`] that holds no field yet.
    #[must_use]
    pub const fn builder() -> ExtractFilingSuperStateBuilder<
        NoFilingRef,
        NoSecClient,
        NoExecutionId,
        NoIngestionMode,
        NoInstanceRepository<I>,
        NoLabelsRepository<L>,
    > {
        ExtractFilingSuperStateBuilder::new()
    }
}

// --- Streaming ---

impl<S, I, L, C> ExtractFilingSuperState<S, I, L, C>
where
    S: State,
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
    Self: IntoStateMachineStream,
{
    // Deviation: `into_run_stream` and `run_to_documents` repeat the run loop of
    // `ExtractNotesSuperState`. Moving both loops into one helper in `stream.rs` changes that
    // super-state too, so the move gets its own PR.
    /// Returns the stream, tagged with the `execution_id` that the context holds.
    ///
    /// Use it instead of `into_stream`, so the stream and the stored documents name the same run.
    #[must_use]
    pub fn into_run_stream(self) -> StateMachineStream {
        let execution_id = self.context.execution_id;
        self.into_stream(execution_id)
    }

    /// Runs the stream to its end and returns the [`FilingDocumentsRef`] of the stored files.
    ///
    /// Calls `on_item` with each successful stream item, in stream order. Each call gets its own
    /// slot for the documents, so clones of one super-state never share it.
    ///
    /// # Errors
    ///
    /// Returns the [`StreamError`] of the first state that fails. The stream ends at that error.
    ///
    /// # Panics
    ///
    /// Panics if the stream ends without an error before [`StoreFilingDocuments`] computes its
    /// output. The state flow rules this out.
    pub async fn run_to_documents(
        mut self,
        mut on_item: impl FnMut(&StreamItem),
    ) -> Result<FilingDocumentsRef, StreamError> {
        let documents_slot = OutputSlot::default();
        self.context.documents_slot = documents_slot.clone();
        let mut stream = self.into_run_stream();
        while let Some(item) = stream.next().await {
            on_item(&item?);
        }
        Ok(documents_slot.take().expect(
            "Given a stream that ended without an error, StoreFilingDocuments has always put the documents into the slot",
        ))
    }
}

impl<I, L, C> NonTerminal for ExtractFilingSuperState<PrepareSecRequest<FilingIndex>, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait + Executes<FilingIndex>,
{
    type Current = PrepareSecRequest<FilingIndex>;
    type Next = ExecuteSecRequest<FilingIndex, C>;
}

impl<I, L, C> NonTerminal for ExtractFilingSuperState<ExecuteSecRequest<FilingIndex, C>, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait + Executes<FilingIndex>,
{
    type Current = ExecuteSecRequest<FilingIndex, C>;
    type Next = SelectFilingDocuments;
}

impl<I, L, C> NonTerminal for ExtractFilingSuperState<SelectFilingDocuments, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    type Current = SelectFilingDocuments;
    type Next = PrepareSecRequest<FilingDocumentSet>;
}

impl<I, L, C> NonTerminal for ExtractFilingSuperState<PrepareSecRequest<FilingDocumentSet>, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait + Executes<FilingDocumentSet>,
{
    type Current = PrepareSecRequest<FilingDocumentSet>;
    type Next = ExecuteSecRequest<FilingDocumentSet, C>;
}

impl<I, L, C> NonTerminal
    for ExtractFilingSuperState<ExecuteSecRequest<FilingDocumentSet, C>, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait + Executes<FilingDocumentSet>,
{
    type Current = ExecuteSecRequest<FilingDocumentSet, C>;
    type Next = StoreFilingDocuments<I, L>;
}

/// Ends the stream after [`StoreFilingDocuments`] writes the files of the filing to the store.
impl<I, L, C> IntoStateMachineStream
    for ExtractFilingSuperState<StoreFilingDocuments<I, L>, I, L, C>
where
    I: RawRepository<RawXbrlInstance> + 'static,
    L: RawRepository<RawXbrlLabels> + 'static,
    C: SecClientTrait + 'static,
{
    fn into_stream(self, execution_id: Uuid) -> StateMachineStream {
        let documents_slot = self.context.documents_slot.clone();
        terminal_stream_with(
            self,
            execution_id,
            move |state: &StoreFilingDocuments<I, L>| {
                if let Some(output) = state.output_data() {
                    documents_slot.put(output.documents.clone());
                }
            },
        )
    }
}

// --- Transitions ---

impl<I, L, C> Transition<PrepareSecRequest<FilingIndex>, ExecuteSecRequest<FilingIndex, C>>
    for ExtractFilingSuperState<PrepareSecRequest<FilingIndex>, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait + Executes<FilingIndex>,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let sec_client = self.context.sec_client.clone();
        Ok(ExtractFilingSuperState {
            current_state: ExecuteSecRequest::try_from((self.current_state, sec_client))?,
            input: ExtractFilingSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<I, L, C> SMTransition<PrepareSecRequest<FilingIndex>, ExecuteSecRequest<FilingIndex, C>>
    for ExtractFilingSuperState<PrepareSecRequest<FilingIndex>, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait + Executes<FilingIndex>,
{
    type NewStateMachine = ExtractFilingSuperState<ExecuteSecRequest<FilingIndex, C>, I, L, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<I, L, C> Transition<ExecuteSecRequest<FilingIndex, C>, SelectFilingDocuments>
    for ExtractFilingSuperState<ExecuteSecRequest<FilingIndex, C>, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait + Executes<FilingIndex>,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        Ok(ExtractFilingSuperState {
            current_state: SelectFilingDocuments::try_from((
                self.current_state,
                self.context.execution_id,
            ))?,
            input: ExtractFilingSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<I, L, C> SMTransition<ExecuteSecRequest<FilingIndex, C>, SelectFilingDocuments>
    for ExtractFilingSuperState<ExecuteSecRequest<FilingIndex, C>, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait + Executes<FilingIndex>,
{
    type NewStateMachine = ExtractFilingSuperState<SelectFilingDocuments, I, L, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<I, L, C> Transition<SelectFilingDocuments, PrepareSecRequest<FilingDocumentSet>>
    for ExtractFilingSuperState<SelectFilingDocuments, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let filing_ref = self.context.filing_ref.clone();
        Ok(ExtractFilingSuperState {
            current_state: PrepareSecRequest::try_from((self.current_state, filing_ref))?,
            input: ExtractFilingSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<I, L, C> SMTransition<SelectFilingDocuments, PrepareSecRequest<FilingDocumentSet>>
    for ExtractFilingSuperState<SelectFilingDocuments, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait,
{
    type NewStateMachine = ExtractFilingSuperState<PrepareSecRequest<FilingDocumentSet>, I, L, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<I, L, C>
    Transition<PrepareSecRequest<FilingDocumentSet>, ExecuteSecRequest<FilingDocumentSet, C>>
    for ExtractFilingSuperState<PrepareSecRequest<FilingDocumentSet>, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait + Executes<FilingDocumentSet>,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let sec_client = self.context.sec_client.clone();
        Ok(ExtractFilingSuperState {
            current_state: ExecuteSecRequest::try_from((self.current_state, sec_client))?,
            input: ExtractFilingSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<I, L, C>
    SMTransition<PrepareSecRequest<FilingDocumentSet>, ExecuteSecRequest<FilingDocumentSet, C>>
    for ExtractFilingSuperState<PrepareSecRequest<FilingDocumentSet>, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait + Executes<FilingDocumentSet>,
{
    type NewStateMachine =
        ExtractFilingSuperState<ExecuteSecRequest<FilingDocumentSet, C>, I, L, C>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<I, L, C> Transition<ExecuteSecRequest<FilingDocumentSet, C>, StoreFilingDocuments<I, L>>
    for ExtractFilingSuperState<ExecuteSecRequest<FilingDocumentSet, C>, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait + Executes<FilingDocumentSet>,
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        Ok(ExtractFilingSuperState {
            current_state: StoreFilingDocuments::try_from((
                self.current_state,
                self.context.execution_id,
                self.context.ingestion_mode,
                self.context.instance_repository.clone(),
                self.context.labels_repository.clone(),
            ))?
            .with_inputs(self.context.inputs.clone()),
            input: ExtractFilingSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<I, L, C> SMTransition<ExecuteSecRequest<FilingDocumentSet, C>, StoreFilingDocuments<I, L>>
    for ExtractFilingSuperState<ExecuteSecRequest<FilingDocumentSet, C>, I, L, C>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
    C: SecClientTrait + Executes<FilingDocumentSet>,
{
    type NewStateMachine = ExtractFilingSuperState<StoreFilingDocuments<I, L>, I, L, C>;

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
    use storage::implementations::repository::filesystem::{Document, FilesystemRepository, Write};
    use tempfile::TempDir;

    use super::*;
    use crate::error::state_machine::transition;
    use crate::implementations::states::extract::execute_sec_request::constants::STATE_NAME as EXECUTE_SEC_REQUEST;
    use crate::implementations::states::extract::prepare_sec_request::constants::STATE_NAME as PREPARE_SEC_REQUEST;
    use crate::implementations::states::extract::prepare_sec_request::{
        PrepareSecRequestContext, PrepareSecRequestInput,
    };
    use crate::implementations::states::extract::select_filing_documents::constants::STATE_NAME as SELECT_FILING_DOCUMENTS;
    use crate::implementations::states::extract::store_filing_documents::constants::STATE_NAME as STORE_FILING_DOCUMENTS;
    use crate::shared::cik::Cik;
    use crate::shared::filing_documents::{FilingDocumentSetRef, FilingDocuments, FolderIndex};
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::raw_document::RawDocument;
    use crate::tests::fixtures::sample_http_client::sample_sec_client::routing::RoutingSecClient;
    use crate::tests::fixtures::sample_write_repository::{
        FakeXbrlInstanceRepository, FakeXbrlLabelsRepository,
    };

    type TestSuperState<S> = ExtractFilingSuperState<
        S,
        FakeXbrlInstanceRepository,
        FakeXbrlLabelsRepository,
        RoutingSecClient,
    >;

    const EXECUTION_ID: Uuid = Uuid::from_u128(7);
    const INSTANCE: &str = "alco-20260908_htm.xml";
    const LABEL_FILE: &str = "alco-20260908_lab.xml";
    const FOLDER_INDEX_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/index.json";
    const INSTANCE_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_htm.xml";
    const LABEL_FILE_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_lab.xml";
    const FOLDER_INDEX: &[u8] = br#"{"directory":{"item":[{"name":"alco-20260908_htm.xml"},{"name":"alco-20260908_lab.xml"}]}}"#;
    const FILING_FOLDER: &str = "sec/filings/CIK0000003545/0000003545-26-000047";

    type InstanceFilesystemRepository = FilesystemRepository<Write, RawDocument<RawXbrlInstance>>;
    type LabelsFilesystemRepository = FilesystemRepository<Write, RawDocument<RawXbrlLabels>>;

    fn sec_client() -> RoutingSecClient {
        RoutingSecClient::new()
            .route(FOLDER_INDEX_URL, 200, "application/json", FOLDER_INDEX)
            .route(INSTANCE_URL, 200, "text/xml", b"<xbrl/>")
            .route(LABEL_FILE_URL, 200, "text/xml", b"<labels/>")
    }

    fn filing_ref() -> FilingRef {
        FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenQ, false),
            NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    fn super_state() -> TestSuperState<PrepareSecRequest<FilingIndex>> {
        ExtractFilingSuperState {
            current_state: PrepareSecRequest::new(
                PrepareSecRequestInput::new(filing_ref()),
                PrepareSecRequestContext::new(filing_ref()),
            ),
            input: ExtractFilingSuperStateData,
            output: None,
            context: ExtractFilingSuperStateContext::new(
                filing_ref(),
                sec_client(),
                EXECUTION_ID,
                IngestionMode::Incremental,
                FakeXbrlInstanceRepository::new(),
                FakeXbrlLabelsRepository::new(),
            ),
        }
    }

    async fn computed<T: State>(mut state: T) -> T {
        let computation: Result<(), StateError> =
            state.compute_output_data_async().await.map_err(Into::into);
        computation.expect(
            "Given a client that serves the folder index and both files, every state should succeed",
        );
        state
    }

    async fn at_folder_index_execute()
    -> TestSuperState<ExecuteSecRequest<FilingIndex, RoutingSecClient>> {
        computed(super_state())
            .await
            .transition_to_next_state_sec()
            .expect("Given a prepared folder index request, the transition should always succeed")
    }

    async fn at_select() -> TestSuperState<SelectFilingDocuments> {
        computed(at_folder_index_execute().await)
            .await
            .transition_to_next_state_sec()
            .expect("Given a served folder index, the transition should always succeed")
    }

    async fn at_documents_prepare() -> TestSuperState<PrepareSecRequest<FilingDocumentSet>> {
        computed(at_select().await)
            .await
            .transition_to_next_state_sec()
            .expect(
                "Given selected documents of the same filing, the transition should always succeed",
            )
    }

    async fn at_documents_execute()
    -> TestSuperState<ExecuteSecRequest<FilingDocumentSet, RoutingSecClient>> {
        computed(at_documents_prepare().await)
            .await
            .transition_to_next_state_sec()
            .expect("Given a prepared documents request, the transition should always succeed")
    }

    fn selected_documents() -> FilingDocumentSetRef {
        FilingDocumentSetRef::new(
            filing_ref(),
            FilingDocuments::new(INSTANCE, Some(LABEL_FILE.to_string())),
        )
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn stored_super_state(
        root: &Path,
        sec_client: RoutingSecClient,
    ) -> ExtractFilingSuperState<
        PrepareSecRequest<FilingIndex>,
        InstanceFilesystemRepository,
        LabelsFilesystemRepository,
        RoutingSecClient,
    > {
        ExtractFilingSuperState::builder()
            .filing_ref(filing_ref())
            .sec_client(sec_client)
            .execution_id(EXECUTION_ID)
            .ingestion_mode(IngestionMode::Incremental)
            .instance_repository(
                FilesystemRepository::builder()
                    .writer()
                    .document_type::<RawDocument<RawXbrlInstance>>()
                    .root(root)
                    .build(),
            )
            .labels_repository(
                FilesystemRepository::builder()
                    .writer()
                    .document_type::<RawDocument<RawXbrlLabels>>()
                    .root(root)
                    .build(),
            )
            .build()
    }

    fn read_stored(root: &Path, file_name: &str) -> Vec<u8> {
        std::fs::read(root.join(FILING_FOLDER).join(file_name))
            .expect("Given a run that stored the file, reading it back should always succeed")
    }

    #[test]
    #[expect(
        clippy::redundant_clone,
        reason = "False positive: `state_name` returns `impl ToString`, which is a `String` here. Clippy flags `.to_string()` as a clone, but removing it does not compile"
    )]
    fn should_return_super_state_name_with_current_state_when_in_prepare_sec_request_state() {
        let super_state = super_state();

        let expected_result = "Extract Filing SuperState (Current: Prepare SEC Request)";

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_zero_max_retries_for_extract_filing_super_state_context() {
        let super_state = super_state();

        let expected_result = 0;

        let result = super_state.context_data().max_retries();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_request_the_folder_index_of_the_filing_when_leaving_the_entry_state() {
        let super_state = computed(super_state()).await;

        let expected_result = &filing_ref();

        let execute = super_state
            .transition_to_next_state_sec()
            .expect("Given a prepared folder index request, the transition should always succeed");
        let result = &execute.current_state().context_data().request_input;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_hand_the_served_folder_index_to_select_filing_documents_when_transitioning() {
        let super_state = computed(at_folder_index_execute().await).await;

        let expected_result = &FolderIndex::new(vec![INSTANCE.to_string(), LABEL_FILE.to_string()]);

        let select = super_state
            .transition_to_next_state_sec()
            .expect("Given a served folder index, the transition should always succeed");
        let result = &select.current_state().input_data().folder_index;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_join_the_filing_ref_of_the_context_and_the_selected_documents_when_transitioning()
     {
        let super_state = computed(at_select().await).await;

        let expected_result = &selected_documents();

        let prepare = super_state.transition_to_next_state_sec().expect(
            "Given selected documents of the same filing, the transition should always succeed",
        );
        let result = prepare.current_state().input_data().request_input();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_request_the_selected_documents_when_leaving_the_documents_prepare_state() {
        let super_state = computed(at_documents_prepare().await).await;

        let expected_result = &selected_documents();

        let execute = super_state
            .transition_to_next_state_sec()
            .expect("Given a prepared documents request, the transition should always succeed");
        let result = &execute.current_state().context_data().request_input;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_pass_the_execution_id_of_the_context_to_the_store_when_transitioning() {
        let super_state = computed(at_documents_execute().await).await;

        let expected_result = EXECUTION_ID;

        let store = super_state
            .transition_to_next_state_sec()
            .expect("Given served documents, the transition should always succeed");
        let result = store.current_state().context_data().execution_id;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_pass_the_execution_id_of_the_context_to_select_filing_documents_when_transitioning()
     {
        let super_state = computed(at_folder_index_execute().await).await;

        let expected_result = EXECUTION_ID;

        let select = super_state
            .transition_to_next_state_sec()
            .expect("Given a served folder index, the transition should always succeed");
        let result = select.current_state().context_data().execution_id;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_leaving_the_entry_state_without_output() {
        let super_state = super_state();

        let expected_result = Err(TransitionError::MissingOutput(
            transition::MissingOutput::new(PREPARE_SEC_REQUEST, EXECUTE_SEC_REQUEST),
        ));

        let result = super_state.transition_to_next_state_sec().map(|_| ());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_served_instance_in_the_filing_folder_when_running_to_the_documents() {
        let root = temporary_root();
        let super_state = stored_super_state(root.path(), sec_client());

        let expected_result = b"<xbrl/>".to_vec();

        super_state
            .run_to_documents(|_| {})
            .await
            .expect("Given a client that serves the folder index and both files, the run should always succeed");
        let result = read_stored(root.path(), "instance.xml");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_served_label_file_in_the_filing_folder_when_running_to_the_documents()
    {
        let root = temporary_root();
        let super_state = stored_super_state(root.path(), sec_client());

        let expected_result = b"<labels/>".to_vec();

        super_state
            .run_to_documents(|_| {})
            .await
            .expect("Given a client that serves the folder index and both files, the run should always succeed");
        let result = read_stored(root.path(), "lab.xml");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_filing_documents_ref_when_running_to_the_documents() {
        let super_state = super_state();

        let expected_result = Some(FilingDocumentsRef::new(filing_ref(), true));

        let result = super_state.run_to_documents(|_| {}).await.ok();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_end_with_the_error_of_select_filing_documents_when_the_folder_has_no_instance()
    {
        let sec_client = RoutingSecClient::new().route(
            FOLDER_INDEX_URL,
            200,
            "application/json",
            br#"{"directory":{"item":[{"name":"alco-20260908_lab.xml"}]}}"#,
        );
        let super_state = ExtractFilingSuperState::builder()
            .filing_ref(filing_ref())
            .sec_client(sec_client)
            .execution_id(EXECUTION_ID)
            .ingestion_mode(IngestionMode::Incremental)
            .instance_repository(FakeXbrlInstanceRepository::new())
            .labels_repository(FakeXbrlLabelsRepository::new())
            .build();

        let expected_result = Some((SELECT_FILING_DOCUMENTS.to_string(), "Problem occurred during internal state operations: '[StateError] A state level error occurred, Caused by: [FailedDocumentSelection] Failure in State: 'Select Filing Documents', Caused by: [MissingXbrlInstance] The folder of filing '0000003545-26-000047' holds no XBRL instance'".to_string()));

        let result = super_state
            .run_to_documents(|_| {})
            .await
            .err()
            .map(|error| (error.state_name, error.source.to_string()));

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_end_with_the_error_of_the_documents_request_when_the_label_file_request_fails()
    {
        let root = temporary_root();
        let sec_client = RoutingSecClient::new()
            .route(FOLDER_INDEX_URL, 200, "application/json", FOLDER_INDEX)
            .route(INSTANCE_URL, 200, "text/xml", b"<xbrl/>");
        let super_state = stored_super_state(root.path(), sec_client);

        let expected_result = Some((EXECUTE_SEC_REQUEST.to_string(), "Problem occurred during internal state operations: '[StateError] A state level error occurred, Caused by: [FailedRequestExecution] Failure in State: 'Execute SEC Request', Caused by: [FailedSecRequest] SEC request failed, Caused by: [FailedFilingDocument] Request for file 'alco-20260908_lab.xml' failed, Caused by: [InvalidResponse] Response validation failed, Caused by: [InvalidSecResponse] Invalid SEC Response, Reason: 'Status code '404' is not accepted''".to_string()));

        let result = super_state
            .run_to_documents(|_| {})
            .await
            .err()
            .map(|error| (error.state_name, error.source.to_string()));

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_no_instance_after_starting_the_documents_request_when_the_label_file_request_fails()
     {
        let root = temporary_root();
        let sec_client = RoutingSecClient::new()
            .route(FOLDER_INDEX_URL, 200, "application/json", FOLDER_INDEX)
            .route(INSTANCE_URL, 200, "text/xml", b"<xbrl/>");
        let super_state = stored_super_state(root.path(), sec_client);

        let expected_result = (5, false);

        let mut started_states = 0;
        let _ = super_state
            .run_to_documents(|item| {
                if item.event == StreamEvent::StateStarted {
                    started_states += 1;
                }
            })
            .await;
        let result = (
            started_states,
            root.path()
                .join(FILING_FOLDER)
                .join("instance.xml")
                .exists(),
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_the_ingestion_mode_of_the_builder_when_storing_the_instance() {
        let ingestion_mode = IngestionMode::Backfill;
        let instance_repository = FakeXbrlInstanceRepository::new();
        let super_state = ExtractFilingSuperState::builder()
            .filing_ref(filing_ref())
            .sec_client(sec_client())
            .execution_id(EXECUTION_ID)
            .ingestion_mode(ingestion_mode)
            .instance_repository(instance_repository.clone())
            .labels_repository(FakeXbrlLabelsRepository::new())
            .build();

        let expected_result = vec![IngestionMode::Backfill];

        super_state.run_to_documents(|_| {}).await.expect(
            "Given a client that serves the folder index and both files, the run should always succeed",
        );
        let result: Vec<IngestionMode> = instance_repository
            .persisted()
            .iter()
            .map(|document| document.metadata().ingestion_mode())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_a_ref_without_label_file_when_the_folder_has_no_label_file() {
        let sec_client = RoutingSecClient::new()
            .route(
                FOLDER_INDEX_URL,
                200,
                "application/json",
                br#"{"directory":{"item":[{"name":"alco-20260908_htm.xml"}]}}"#,
            )
            .route(INSTANCE_URL, 200, "text/xml", b"<xbrl/>");
        let super_state = ExtractFilingSuperState::builder()
            .filing_ref(filing_ref())
            .sec_client(sec_client)
            .execution_id(EXECUTION_ID)
            .ingestion_mode(IngestionMode::Incremental)
            .instance_repository(FakeXbrlInstanceRepository::new())
            .labels_repository(FakeXbrlLabelsRepository::new())
            .build();

        let expected_result = Some(FilingDocumentsRef::new(filing_ref(), false));

        let result = super_state.run_to_documents(|_| {}).await.ok();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_hand_the_events_of_every_state_in_order_to_the_caller_when_running_to_the_documents()
     {
        let super_state = super_state();

        let expected_result = vec![
            (StreamEvent::StateStarted, PREPARE_SEC_REQUEST),
            (StreamEvent::StateCompleted, PREPARE_SEC_REQUEST),
            (StreamEvent::TransitionCompleted, PREPARE_SEC_REQUEST),
            (StreamEvent::StateStarted, EXECUTE_SEC_REQUEST),
            (StreamEvent::StateCompleted, EXECUTE_SEC_REQUEST),
            (StreamEvent::TransitionCompleted, EXECUTE_SEC_REQUEST),
            (StreamEvent::StateStarted, SELECT_FILING_DOCUMENTS),
            (StreamEvent::StateCompleted, SELECT_FILING_DOCUMENTS),
            (StreamEvent::TransitionCompleted, SELECT_FILING_DOCUMENTS),
            (StreamEvent::StateStarted, PREPARE_SEC_REQUEST),
            (StreamEvent::StateCompleted, PREPARE_SEC_REQUEST),
            (StreamEvent::TransitionCompleted, PREPARE_SEC_REQUEST),
            (StreamEvent::StateStarted, EXECUTE_SEC_REQUEST),
            (StreamEvent::StateCompleted, EXECUTE_SEC_REQUEST),
            (StreamEvent::TransitionCompleted, EXECUTE_SEC_REQUEST),
            (StreamEvent::StateStarted, STORE_FILING_DOCUMENTS),
            (StreamEvent::StateCompleted, STORE_FILING_DOCUMENTS),
        ];

        let mut events = Vec::new();
        super_state
            .run_to_documents(|item| events.push((item.event.clone(), item.state_name.clone())))
            .await
            .expect("Given a client that serves the folder index and both files, the run should always succeed");
        let result: Vec<(StreamEvent, &str)> = events
            .iter()
            .map(|(event, state_name)| (event.clone(), state_name.as_str()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_documents_when_a_clone_empties_the_slot_it_was_built_with() {
        let super_state = super_state();
        let built_slot = super_state.context.documents_slot.clone();

        let expected_result = Some(FilingDocumentsRef::new(filing_ref(), true));

        let result = super_state
            .run_to_documents(|_| {
                built_slot.take();
            })
            .await
            .ok();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_extract_filing_super_state() {
        implements_auto_traits::<TestSuperState<PrepareSecRequest<FilingIndex>>>();
    }

    #[test]
    const fn should_implement_auto_traits_for_extract_filing_super_state_context() {
        implements_auto_traits::<
            ExtractFilingSuperStateContext<
                FakeXbrlInstanceRepository,
                FakeXbrlLabelsRepository,
                RoutingSecClient,
            >,
        >();
    }

    #[test]
    const fn should_implement_auto_traits_for_extract_filing_super_state_data() {
        implements_auto_traits::<ExtractFilingSuperStateData>();
    }
}
