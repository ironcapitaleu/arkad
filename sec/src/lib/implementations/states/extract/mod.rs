//! # Extract Phase
//!
//! Provides the states of the extract phase and the [`ExtractSuperState`] that drives them.
//!
//! Extraction turns a raw CIK string into a fetched SEC response through three ordered
//! states. The super-state owns the shared HTTP client, the `execution_id` of the run, and the
//! storage repository. It also owns the type-safe transitions between the states, so callers
//! advance the pipeline without managing each state by hand. A fourth state,
//! [`StoreCompanyFacts`], writes a fetched response to the store. A transition leads into it from
//! [`ExecuteSecRequest`], but the stream does not run it.
//!
//! ## Modules
//!
//! - [`validate_cik_format`]: Validates and normalizes the raw CIK.
//! - [`prepare_sec_request`]: Builds the request targeting the SEC API endpoint.
//! - [`execute_sec_request`]: Sends the request and captures the response.
//! - [`store_company_facts`]: Writes the raw body of the response to the store.
//!
//! ## State Flow
//!
//! [`ValidateCikFormat`] → [`PrepareSecRequest`] → [`ExecuteSecRequest`], after which the
//! stream crosses into the transform phase. [`ExecuteSecRequest`] can also transition to
//! [`StoreCompanyFacts`].
//!
//! ## Usage
//!
//! ```no_run
//! use sec::implementations::states::extract::*;
//! use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
//! use sec::prelude::*;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use sec::shared::raw_document::RawDocument;
//! use sec::shared::raw_source::RawCompanyFacts;
//! use storage::implementations::repository::filesystem::FilesystemRepository;
//! use uuid::Uuid;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let repository = FilesystemRepository::builder()
//!     .writer()
//!     .document_type::<RawDocument<RawCompanyFacts>>()
//!     .root("data")
//!     .build();
//! let mut extract_state = ExtractSuperState::<ValidateCikFormat, _>::new(
//!     "1234567890",
//!     SecClient::default(),
//!     Uuid::new_v4(),
//!     repository,
//! );
//! extract_state.compute_output_data_async().await?;
//! let _next_state = extract_state.transition_to_next_state_sec()?;
//! # Ok(())
//! # }
//! ```

pub mod execute_sec_request;
pub mod prepare_sec_request;
pub mod store_company_facts;
pub mod validate_cik_format;

use std::fmt::{self, Display, Formatter};

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::{StateMachine as SMStateMachine, Transition as SMTransition};
use uuid::Uuid;

use crate::error::State as StateError;
use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::constants::STATE_NAME as EXECUTE_SEC_REQUEST;
use crate::implementations::states::extract::execute_sec_request::{
    ExecuteSecRequest, ExecuteSecRequestContext, ExecuteSecRequestInput,
};
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use crate::implementations::states::extract::store_company_facts::{
    RawCompanyFactsRepository, StoreCompanyFacts, StoreCompanyFactsContext, StoreCompanyFactsInput,
};
use crate::implementations::states::extract::validate_cik_format::{
    ValidateCikFormat, ValidateCikFormatContext, ValidateCikFormatInput,
};
use crate::implementations::states::transform::TransformSuperState;
use crate::implementations::states::transform::parse_company_facts::ParseCompanyFacts;
use crate::implementations::states::transform::parse_company_facts::constants::STATE_NAME as PARSE_COMPANY_FACTS;
use crate::prelude::*;
use crate::shared::cik::Cik;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;

/// Input and output data for the [`ExtractSuperState`].
///
/// A placeholder with unit update semantics: the super-state delegates all real data
/// handling to its current inner state, so it carries no data of its own.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ExtractSuperStateData;

impl SMStateData for ExtractSuperStateData {
    type UpdateType = ();
    fn state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, (): Self::UpdateType) {}
}

impl StateData for ExtractSuperStateData {
    fn update_state(&mut self, (): Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

/// Context for the [`ExtractSuperState`].
///
/// Holds the shared [`SecClient`] handed down to each inner state, so the client is
/// constructed once and reused across the whole extract phase. It also holds the
/// `execution_id` of the run and the storage repository `R` that [`StoreCompanyFacts`] writes to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ExtractSuperStateContext<R> {
    pub(crate) sec_client: SecClient,
    pub(crate) execution_id: Uuid,
    pub(crate) repository: R,
}

impl<R> ExtractSuperStateContext<R> {
    /// Creates a new [`ExtractSuperStateContext`] from the shared HTTP client, the
    /// `execution_id` of the run, and the storage repository.
    #[must_use]
    pub const fn new(sec_client: SecClient, execution_id: Uuid, repository: R) -> Self {
        Self {
            sec_client,
            execution_id,
            repository,
        }
    }
}

impl<R: RawCompanyFactsRepository> SMContext for ExtractSuperStateContext<R> {
    type UpdateType = ();
    fn context(&self) -> &Self {
        self
    }
    fn update_context(&mut self, (): Self::UpdateType) {}
}

impl<R: RawCompanyFactsRepository> Context for ExtractSuperStateContext<R> {
    fn max_retries(&self) -> u32 {
        0
    }
}

/// Hierarchical super-state that orchestrates the extract phase.
///
/// Wraps the currently active extract state in its `S` type parameter and owns the shared
/// HTTP client, the `execution_id` of the run, and the storage repository `R`. It exposes the
/// inner state through a unified state-machine interface. It moves the pipeline through
/// type-safe transitions: [`ValidateCikFormat`] → [`PrepareSecRequest`] →
/// [`ExecuteSecRequest`]. [`ExecuteSecRequest`] can also transition to [`StoreCompanyFacts`].
/// Encoding the active state in the type makes invalid transitions a compile error rather than a
/// runtime check.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ExtractSuperState<S: State, R: RawCompanyFactsRepository> {
    current_state: S,
    input: ExtractSuperStateData,
    output: Option<ExtractSuperStateData>,
    context: ExtractSuperStateContext<R>,
}

impl<S: State, R: RawCompanyFactsRepository> Display for ExtractSuperState<S, R> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Extract SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }
}

impl<S: State, R: RawCompanyFactsRepository> SMState for ExtractSuperState<S, R> {
    type InputData = ExtractSuperStateData;
    type OutputData = ExtractSuperStateData;
    type Context = ExtractSuperStateContext<R>;

    fn state_name(&self) -> impl ToString {
        format!(
            "Extract SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }
    fn input_data(&self) -> &Self::InputData {
        &self.input
    }
    /// Blocking wrapper around [`compute_output_data_async`](crate::traits::state_machine::state::State::compute_output_data_async).
    ///
    /// Detects whether a tokio runtime is available and runs the async computation
    /// synchronously. This allows SEC states to be used as regular `SMState` implementations.
    ///
    /// # Panics
    /// - Panics if the async computation returns an error.
    /// - Panics if called inside a `current_thread` runtime (`block_in_place`
    ///   requires a multi-thread runtime). All binaries and tests in this project
    ///   use `flavor = "multi_thread"`.
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
            #[allow(clippy::useless_conversion)]
            let state_err: crate::error::State = e.into();
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
impl<S: State, R: RawCompanyFactsRepository> State for ExtractSuperState<S, R> {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.current_state
            .compute_output_data_async()
            .await
            .map_err(std::convert::Into::into)
    }
}

impl<S: State, R: RawCompanyFactsRepository> SMStateMachine<S> for ExtractSuperState<S, R> {
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

impl<S: State, R: RawCompanyFactsRepository> StateMachine<S> for ExtractSuperState<S, R> {}

impl<S: State, R: RawCompanyFactsRepository> SMSuperState<S> for ExtractSuperState<S, R> {}

impl<S: State, R: RawCompanyFactsRepository> SuperState<S> for ExtractSuperState<S, R> {}

impl<R: RawCompanyFactsRepository> ExtractSuperState<ValidateCikFormat, R> {
    /// Creates the super-state at the pipeline entry point from a raw CIK, the shared client, the
    /// `execution_id` of the run, and the storage repository.
    ///
    /// [`ExtractSuperState::into_run_stream`] tags the stream with `execution_id`.
    #[must_use]
    pub fn new(
        input: impl Into<String>,
        sec_client: SecClient,
        execution_id: Uuid,
        repository: R,
    ) -> Self {
        let input: String = input.into();
        let input_data = ValidateCikFormatInput::new(input.clone());
        let state_context = ValidateCikFormatContext::new(input, sec_client.clone());

        Self {
            current_state: ValidateCikFormat::new(input_data, state_context),
            input: ExtractSuperStateData,
            output: None,
            context: ExtractSuperStateContext::new(sec_client, execution_id, repository),
        }
    }
}

impl<R: RawCompanyFactsRepository> ExtractSuperState<PrepareSecRequest, R> {
    /// Creates the super-state positioned at the request-preparation state.
    #[must_use]
    pub fn new(
        validated_cik: Cik,
        sec_client: SecClient,
        execution_id: Uuid,
        repository: R,
    ) -> Self {
        let input_data = PrepareSecRequestInput::new(validated_cik.clone(), sec_client.clone());
        let context = PrepareSecRequestContext::new(validated_cik);

        Self {
            current_state: PrepareSecRequest::new(input_data, context),
            input: ExtractSuperStateData,
            output: None,
            context: ExtractSuperStateContext::new(sec_client, execution_id, repository),
        }
    }
}

impl<R: RawCompanyFactsRepository> ExtractSuperState<ExecuteSecRequest, R> {
    /// Creates the super-state positioned at the request-execution state.
    #[must_use]
    pub fn new(
        client: SecClient,
        request: SecRequest,
        cik: Cik,
        execution_id: Uuid,
        repository: R,
    ) -> Self {
        let esr_input = ExecuteSecRequestInput::new(client.clone(), request);
        let esr_context = ExecuteSecRequestContext::new(cik);

        Self {
            current_state: ExecuteSecRequest::new(esr_input, esr_context),
            input: ExtractSuperStateData,
            output: None,
            context: ExtractSuperStateContext::new(client, execution_id, repository),
        }
    }
}

impl<R: RawCompanyFactsRepository> ExtractSuperState<StoreCompanyFacts<R>, R> {
    /// Creates the super-state positioned at the state that stores the Company Facts.
    ///
    /// The metadata of the stored document records `execution_id`.
    #[must_use]
    pub fn new(
        client: SecClient,
        response: SecResponse,
        requested_url: Url,
        user_agent: UserAgent,
        cik: Cik,
        execution_id: Uuid,
        repository: R,
    ) -> Self {
        let input = StoreCompanyFactsInput::new(response, requested_url, user_agent);
        let context = StoreCompanyFactsContext::new(cik, execution_id, repository.clone());

        Self {
            current_state: StoreCompanyFacts::new(input, context),
            input: ExtractSuperStateData,
            output: None,
            context: ExtractSuperStateContext::new(client, execution_id, repository),
        }
    }
}

impl<S: State, R: RawCompanyFactsRepository> ExtractSuperState<S, R> {
    /// Consumes the `SuperState` and returns the inner state.
    #[must_use]
    pub fn into_current_state(self) -> S {
        self.current_state
    }
}

// --- Streaming ---

impl<S: State, R: RawCompanyFactsRepository> ExtractSuperState<S, R>
where
    Self: IntoStateMachineStream,
{
    /// Returns the stream of the run, tagged with the `execution_id` that the context holds.
    ///
    /// Use it instead of `into_stream`, so the stream and the stored document name the same run.
    #[must_use]
    pub fn into_run_stream(self) -> StateMachineStream {
        let execution_id = self.context.execution_id;
        self.into_stream(execution_id)
    }
}

impl<R: RawCompanyFactsRepository> NonTerminal for ExtractSuperState<ValidateCikFormat, R> {
    type Current = ValidateCikFormat;
    type Next = PrepareSecRequest;
}

impl<R: RawCompanyFactsRepository> NonTerminal for ExtractSuperState<PrepareSecRequest, R> {
    type Current = PrepareSecRequest;
    type Next = ExecuteSecRequest;
}

/// `ExecuteSecRequest` is no longer terminal -- it transitions into the Transform `SuperState`.
impl<R: RawCompanyFactsRepository> NonTerminal for ExtractSuperState<ExecuteSecRequest, R> {
    type Current = ExecuteSecRequest;
    type Next = ParseCompanyFacts;
}

impl<R: RawCompanyFactsRepository> Transition<ValidateCikFormat, PrepareSecRequest>
    for ExtractSuperState<ValidateCikFormat, R>
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let next_state = PrepareSecRequest::try_from(self.current_state)?;

        Ok(ExtractSuperState::<PrepareSecRequest, R> {
            current_state: next_state,
            input: ExtractSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R: RawCompanyFactsRepository> Transition<PrepareSecRequest, ExecuteSecRequest>
    for ExtractSuperState<PrepareSecRequest, R>
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let next_state = ExecuteSecRequest::try_from(self.current_state)?;

        Ok(ExtractSuperState::<ExecuteSecRequest, R> {
            current_state: next_state,
            input: ExtractSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R: RawCompanyFactsRepository> SMTransition<PrepareSecRequest, ExecuteSecRequest>
    for ExtractSuperState<PrepareSecRequest, R>
{
    type NewStateMachine = ExtractSuperState<ExecuteSecRequest, R>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        // Placeholder implementation - use transition_to_next_state_sec() for actual functionality
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<R: RawCompanyFactsRepository> SMTransition<ValidateCikFormat, PrepareSecRequest>
    for ExtractSuperState<ValidateCikFormat, R>
{
    type NewStateMachine = ExtractSuperState<PrepareSecRequest, R>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        // Placeholder implementation - use transition_to_next_state_sec() for actual functionality
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

// --- Cross-SuperState transition: Extract → Transform ---

impl<R: RawCompanyFactsRepository> Transition<ExecuteSecRequest, ParseCompanyFacts>
    for ExtractSuperState<ExecuteSecRequest, R>
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let inner_state = self.into_current_state();
        let (_input, output, context) = inner_state.into_parts();

        let output_data = output.ok_or_else(|| {
            transition::MissingOutput::new(EXECUTE_SEC_REQUEST, PARSE_COMPANY_FACTS)
        })?;

        let sec_response = output_data.response;
        let cik = context.cik;

        Ok(TransformSuperState::<ParseCompanyFacts>::new(
            &sec_response,
            cik,
        ))
    }
}

impl<R: RawCompanyFactsRepository> SMTransition<ExecuteSecRequest, ParseCompanyFacts>
    for ExtractSuperState<ExecuteSecRequest, R>
{
    type NewStateMachine = TransformSuperState<ParseCompanyFacts>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl<R: RawCompanyFactsRepository> Transition<ExecuteSecRequest, StoreCompanyFacts<R>>
    for ExtractSuperState<ExecuteSecRequest, R>
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let next_state = StoreCompanyFacts::try_from_execute_sec_request(
            self.current_state,
            self.context.execution_id,
            self.context.repository.clone(),
        )?;

        Ok(ExtractSuperState::<StoreCompanyFacts<R>, R> {
            current_state: next_state,
            input: ExtractSuperStateData,
            output: None,
            context: self.context,
        })
    }
}

impl<R: RawCompanyFactsRepository> SMTransition<ExecuteSecRequest, StoreCompanyFacts<R>>
    for ExtractSuperState<ExecuteSecRequest, R>
{
    type NewStateMachine = ExtractSuperState<StoreCompanyFacts<R>, R>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::{fmt::Debug, hash::Hash};

    use futures_util::StreamExt;
    use pretty_assertions::assert_eq;
    use storage::implementations::repository::filesystem::Document;

    use super::*;
    use crate::implementations::states::extract::execute_sec_request::ExecuteSecRequestOutput;
    use crate::implementations::states::extract::store_company_facts::STATE_NAME as STORE_COMPANY_FACTS;
    use crate::shared::cik::Cik;
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::http_client::implementations::sec_client::SecClient;
    use crate::shared::status_code::StatusCode;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    #[test]
    #[expect(
        clippy::redundant_clone,
        reason = "False positive: `state_name` returns `impl ToString`, which is a `String` here. Clippy flags `.to_string()` as a clone, but removing it does not compile"
    )]
    fn should_return_super_state_name_with_current_state_when_in_validate_cik_format_state() {
        let input_cik = "1234567890";
        let sec_client = SecClient::default();
        let super_state = ExtractSuperState::<ValidateCikFormat, _>::new(
            input_cik,
            sec_client,
            Uuid::nil(),
            FakeWriteRepository::new(),
        );

        let expected_result = "Extract SuperState (Current: Validate CIK Format)";

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[expect(
        clippy::redundant_clone,
        reason = "False positive: `state_name` returns `impl ToString`, which is a `String` here. Clippy flags `.to_string()` as a clone, but removing it does not compile"
    )]
    fn should_return_super_state_name_with_current_state_when_in_prepare_sec_request_state() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should be valid");
        let sec_client = SecClient::default();
        let super_state = ExtractSuperState::<PrepareSecRequest, _>::new(
            cik,
            sec_client,
            Uuid::nil(),
            FakeWriteRepository::new(),
        );

        let expected_result = "Extract SuperState (Current: Prepare SEC Request)";

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_access_current_validate_cik_format_state_from_super_state() {
        let input_cik = "1234567890";
        let sec_client = SecClient::default();
        let super_state = ExtractSuperState::<ValidateCikFormat, _>::new(
            input_cik,
            sec_client,
            Uuid::nil(),
            FakeWriteRepository::new(),
        );

        let expected_result = "Validate CIK Format";

        let result = super_state.current_state().state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_access_current_prepare_sec_request_state_from_super_state() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should be valid");
        let sec_client = SecClient::default();
        let super_state = ExtractSuperState::<PrepareSecRequest, _>::new(
            cik,
            sec_client,
            Uuid::nil(),
            FakeWriteRepository::new(),
        );

        let expected_result = "Prepare SEC Request";

        let result = super_state.current_state().state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    #[expect(
        clippy::redundant_clone,
        reason = "False positive: `state_name` returns `impl ToString`, which is a `String` here. Clippy flags `.to_string()` as a clone, but removing it does not compile"
    )]
    async fn should_transition_from_prepare_sec_request_to_execute_sec_request_state() {
        let input_cik = "1234567890";
        let sec_client = SecClient::default();
        let mut super_state = ExtractSuperState::<ValidateCikFormat, _>::new(
            input_cik,
            sec_client,
            Uuid::nil(),
            FakeWriteRepository::new(),
        );

        super_state
            .compute_output_data_async()
            .await
            .expect("Should compute output data");

        let mut super_state = super_state
            .transition_to_next_state_sec()
            .expect("Should transition to PrepareSecRequest");

        super_state
            .compute_output_data_async()
            .await
            .expect("Should compute output data");

        let expected_result = "Extract SuperState (Current: Execute SEC Request)";

        let state = super_state
            .transition_to_next_state_sec()
            .expect("Should transition to ExecuteSecRequest");

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_transition_from_prepare_sec_request_when_output_data_not_yet_computed() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should be valid");
        let sec_client = SecClient::default();
        let super_state = ExtractSuperState::<PrepareSecRequest, _>::new(
            cik,
            sec_client,
            Uuid::nil(),
            FakeWriteRepository::new(),
        );

        let expected_result = true;
        let result = super_state.transition_to_next_state_sec().is_err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_delegate_computation_to_current_state_when_computing_output_data() {
        let input_cik = "1234567890";
        let sec_client = SecClient::default();
        let mut super_state = ExtractSuperState::<ValidateCikFormat, _>::new(
            input_cik,
            sec_client,
            Uuid::nil(),
            FakeWriteRepository::new(),
        );

        let expected_result = Ok(());

        let result = super_state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    #[expect(
        clippy::redundant_clone,
        reason = "False positive: `state_name` returns `impl ToString`, which is a `String` here. Clippy flags `.to_string()` as a clone, but removing it does not compile"
    )]
    async fn should_transition_from_validate_cik_format_to_prepare_sec_request_state() {
        let input_cik = "1234567890";
        let sec_client = SecClient::default();
        let mut super_state = ExtractSuperState::<ValidateCikFormat, _>::new(
            input_cik,
            sec_client,
            Uuid::nil(),
            FakeWriteRepository::new(),
        );

        super_state
            .compute_output_data_async()
            .await
            .expect("Should compute output data");

        let expected_result = "Extract SuperState (Current: Prepare SEC Request)";

        let result = super_state
            .transition_to_next_state_sec()
            .unwrap()
            .state_name()
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_transition_from_validate_cik_format_when_output_data_not_yet_computed() {
        let input_cik = "1234567890";
        let sec_client = SecClient::default();
        let super_state = ExtractSuperState::<ValidateCikFormat, _>::new(
            input_cik,
            sec_client,
            Uuid::nil(),
            FakeWriteRepository::new(),
        );

        let expected_result = true;

        let result = super_state.transition_to_next_state_sec().is_err();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_validate_cik_format_super_state() {
        implements_auto_traits::<ExtractSuperState<ValidateCikFormat, FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_auto_traits_for_prepare_sec_request_super_state() {
        implements_auto_traits::<ExtractSuperState<PrepareSecRequest, FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_auto_traits_for_execute_sec_request_super_state() {
        implements_auto_traits::<ExtractSuperState<ExecuteSecRequest, FakeWriteRepository>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_be_thread_safe_for_validate_cik_format_super_state() {
        implements_send::<ExtractSuperState<ValidateCikFormat, FakeWriteRepository>>();
        implements_sync::<ExtractSuperState<ValidateCikFormat, FakeWriteRepository>>();
    }

    #[test]
    const fn should_be_thread_safe_for_prepare_sec_request_super_state() {
        implements_send::<ExtractSuperState<PrepareSecRequest, FakeWriteRepository>>();
        implements_sync::<ExtractSuperState<PrepareSecRequest, FakeWriteRepository>>();
    }

    #[test]
    const fn should_be_thread_safe_for_execute_sec_request_super_state() {
        implements_send::<ExtractSuperState<ExecuteSecRequest, FakeWriteRepository>>();
        implements_sync::<ExtractSuperState<ExecuteSecRequest, FakeWriteRepository>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_validate_cik_format_super_state() {
        implements_debug::<ExtractSuperState<ValidateCikFormat, FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_debug_for_prepare_sec_request_super_state() {
        implements_debug::<ExtractSuperState<PrepareSecRequest, FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_debug_for_execute_sec_request_super_state() {
        implements_debug::<ExtractSuperState<ExecuteSecRequest, FakeWriteRepository>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_validate_cik_format_super_state() {
        implements_clone::<ExtractSuperState<ValidateCikFormat, FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_clone_for_prepare_sec_request_super_state() {
        implements_clone::<ExtractSuperState<PrepareSecRequest, FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_clone_for_execute_sec_request_super_state() {
        implements_clone::<ExtractSuperState<ExecuteSecRequest, FakeWriteRepository>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_validate_cik_format_super_state() {
        implements_partial_eq::<ExtractSuperState<ValidateCikFormat, FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_partial_eq_for_prepare_sec_request_super_state() {
        implements_partial_eq::<ExtractSuperState<PrepareSecRequest, FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_partial_eq_for_execute_sec_request_super_state() {
        implements_partial_eq::<ExtractSuperState<ExecuteSecRequest, FakeWriteRepository>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_validate_cik_format_super_state() {
        implements_hash::<ExtractSuperState<ValidateCikFormat, FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_hash_for_prepare_sec_request_super_state() {
        implements_hash::<ExtractSuperState<PrepareSecRequest, FakeWriteRepository>>();
    }

    #[test]
    const fn should_implement_hash_for_execute_sec_request_super_state() {
        implements_hash::<ExtractSuperState<ExecuteSecRequest, FakeWriteRepository>>();
    }

    fn sample_url() -> Url {
        "https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn sample_response() -> SecResponse {
        SecResponse::from_parts(
            sample_url(),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            b"{}".to_vec(),
        )
        .expect("Given the hardcoded JSON body '{}', building the response should always succeed")
    }

    fn store_company_facts_super_state(
        execution_id: Uuid,
    ) -> ExtractSuperState<StoreCompanyFacts<FakeWriteRepository>, FakeWriteRepository> {
        let url = sample_url();
        let response = sample_response();
        let user_agent = UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        let cik = Cik::new("1067983")
            .expect("Given a hardcoded numeric CIK, its creation should always succeed");
        ExtractSuperState::<StoreCompanyFacts<FakeWriteRepository>, _>::new(
            SecClient::default(),
            response,
            url,
            user_agent,
            cik,
            execution_id,
            FakeWriteRepository::new(),
        )
    }

    #[tokio::test]
    async fn should_record_the_execution_id_of_the_constructor_when_storing_company_facts() {
        let execution_id = Uuid::from_u128(7);
        let mut super_state = store_company_facts_super_state(execution_id);
        let repository = super_state
            .current_state()
            .context_data()
            .repository()
            .clone();

        let expected_result = vec![execution_id];

        super_state
            .compute_output_data_async()
            .await
            .expect("Given a fake repository that accepts every write, the state should succeed");
        let result: Vec<Uuid> = repository
            .persisted()
            .iter()
            .map(|document| document.metadata().execution_id())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_execution_id_of_first_constructor_when_transitioning_to_execute_sec_request()
     {
        let execution_id = Uuid::from_u128(7);
        let mut super_state = ExtractSuperState::<ValidateCikFormat, _>::new(
            "1067983",
            SecClient::default(),
            execution_id,
            FakeWriteRepository::new(),
        );
        super_state
            .compute_output_data_async()
            .await
            .expect("Given a hardcoded valid CIK, validation should always succeed");
        let mut super_state = super_state
            .transition_to_next_state_sec()
            .expect("Given a computed CIK validation, the transition should always succeed");
        super_state
            .compute_output_data_async()
            .await
            .expect("Given a validated CIK, preparing the request should always succeed");

        let expected_result = execution_id;

        let result = super_state
            .transition_to_next_state_sec()
            .expect("Given a prepared request, the transition should always succeed")
            .context_data()
            .execution_id;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_carry_execution_id_of_first_constructor_when_transitioning_to_store_company_facts()
     {
        let execution_id = Uuid::from_u128(7);
        let mut super_state = ExtractSuperState::<ValidateCikFormat, _>::new(
            "1067983",
            SecClient::default(),
            execution_id,
            FakeWriteRepository::new(),
        );
        super_state
            .compute_output_data_async()
            .await
            .expect("Given a hardcoded valid CIK, validation should always succeed");
        let mut super_state = super_state
            .transition_to_next_state_sec()
            .expect("Given a computed CIK validation, the transition should always succeed");
        super_state
            .compute_output_data_async()
            .await
            .expect("Given a validated CIK, preparing the request should always succeed");
        let mut super_state = super_state
            .transition_to_next_state_sec()
            .expect("Given a prepared request, the transition should always succeed");
        let executed = super_state
            .current_state()
            .clone()
            .with_output(ExecuteSecRequestOutput::new(sample_response()));
        *super_state.current_state_mut() = executed;

        let expected_result = execution_id;

        let result = Transition::<ExecuteSecRequest, StoreCompanyFacts<FakeWriteRepository>>::transition_to_next_state_sec(super_state)
            .expect("Given a request with output, the transition should always succeed")
            .current_state()
            .context_data()
            .execution_id();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_output_when_transitioning_to_store_company_facts_without_output() {
        let cik = Cik::new("1067983")
            .expect("Given a hardcoded numeric CIK, its creation should always succeed");
        let request = SecRequest::builder()
            .all_company_facts()
            .cik(cik.clone())
            .build();
        let super_state = ExtractSuperState::<ExecuteSecRequest, _>::new(
            SecClient::default(),
            request,
            cik,
            Uuid::nil(),
            FakeWriteRepository::new(),
        );

        let expected_result = TransitionError::MissingOutput(transition::MissingOutput::new(
            EXECUTE_SEC_REQUEST,
            STORE_COMPANY_FACTS,
        ));

        let result = Transition::<ExecuteSecRequest, StoreCompanyFacts<FakeWriteRepository>>::transition_to_next_state_sec(super_state)
            .expect_err("Given a request without output, the transition should always fail");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_tag_stream_with_execution_id_of_context_when_streaming_the_run() {
        let execution_id = Uuid::from_u128(7);
        let super_state = ExtractSuperState::<ValidateCikFormat, _>::new(
            "ABC",
            SecClient::default(),
            execution_id,
            FakeWriteRepository::new(),
        );
        let mut stream = super_state.into_run_stream();

        let expected_result = Some(execution_id);

        let mut result = None;
        while let Some(item) = stream.next().await {
            if let Err(error) = item {
                result = Some(error.execution_id);
            }
        }

        assert_eq!(result, expected_result);
    }

    #[test]
    const fn should_implement_auto_traits_for_store_company_facts_super_state() {
        implements_auto_traits::<
            ExtractSuperState<StoreCompanyFacts<FakeWriteRepository>, FakeWriteRepository>,
        >();
    }
}
