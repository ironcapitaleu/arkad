//! # Extract State Module
//!
//! This module provides state implementations for the extraction phase of the SEC filings ETL workflow.
//! It handles validation of raw input data and preparation of SEC API requests through a hierarchical super-state.
//!
//! ## Components
//! - [`validate_cik_format`]: Validates and normalizes CIK (Central Index Key) strings to proper 10-digit format.
//! - [`prepare_sec_request`]: Creates HTTP clients and prepares request objects for SEC API calls.
//! - [`execute_sec_request`]: Executes prepared SEC API requests and handles responses.
//! - [`ExtractSuperState`]: Super-state that orchestrates the extraction workflow and state transitions.
//!
//! ## State Flow
//! The extraction follows this progression: [`ValidateCikFormat`] → [`PrepareSecRequest`] → [`ExecuteSecRequest`]
//!
//! ## Example
//! ```rust
//! use sec::implementations::states::extract::*;
//! use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
//! use sec::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut extract_state = ExtractSuperState::<ValidateCikFormat>::new("1234567890");
//!     extract_state.compute_output_data_async().await?;
//!     let next_state = extract_state.transition_to_next_state_sec()?;
//!     Ok(())
//! }
//! ```

pub mod execute_sec_request;
pub mod prepare_sec_request;
pub mod validate_cik_format;

use std::fmt::Display;

use crate::error::State as StateError;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::{
    ExecuteSecRequest, ExecuteSecRequestContext, ExecuteSecRequestInput,
};
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use crate::implementations::states::extract::validate_cik_format::{
    ValidateCikFormat, ValidateCikFormatContext, ValidateCikFormatInput,
};

use crate::shared::cik::Cik;
use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::shared::request::implementations::sec_request::SecRequest;

use async_trait::async_trait;

use crate::prelude::*;
use state_maschine::prelude::{StateMachine as SMStateMachine, Transition as SMTransition};

/// Data structure for the Extract super-state.
///
/// Currently serves as a placeholder type with unit update semantics for the [`ExtractSuperState`].
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
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

/// Context data structure for the Extract super-state.
///
/// Provides configuration and runtime context for the [`ExtractSuperState`], including retry policies.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct ExtractSuperStateContext;

impl SMContext for ExtractSuperStateContext {
    type UpdateType = ();
    fn context(&self) -> &Self {
        self
    }
    fn update_context(&mut self, (): Self::UpdateType) {}
}

impl Context for ExtractSuperStateContext {
    fn max_retries(&self) -> u32 {
        0
    }
}

/// A hierarchical super-state that orchestrates the extraction phase of the SEC ETL pipeline.
///
/// Manages progression through extraction states like [`ValidateCikFormat`] and [`PrepareSecRequest`],
/// providing type-safe transitions and unified state machine interfaces.
///
/// # Type Parameter
/// - `S`: The current active state, which must implement the [`State`] trait
///
/// # State Transitions
/// Supports transitions: `ValidateCikFormat` → `PrepareSecRequest`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct ExtractSuperState<S: State> {
    current_state: S,
    input: ExtractSuperStateData,
    output: Option<ExtractSuperStateData>,
    context: ExtractSuperStateContext,
}

impl<S: State> Display for ExtractSuperState<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Extract SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }
}

impl<S: State> SMState for ExtractSuperState<S> {
    type InputData = ExtractSuperStateData;
    type OutputData = ExtractSuperStateData;
    type Context = ExtractSuperStateContext;

    fn state_name(&self) -> impl ToString {
        format!(
            "Extract SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }
    fn input_data(&self) -> &Self::InputData {
        &self.input
    }
    fn compute_output_data(&mut self) { /* handled by async version */
    }
    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
    fn context_data(&self) -> &Self::Context {
        &self.context
    }
}

#[async_trait]
impl<S: State> State for ExtractSuperState<S> {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.current_state
            .compute_output_data_async()
            .await
            .map_err(std::convert::Into::into)
    }
}

impl<S: State> SMStateMachine<S> for ExtractSuperState<S> {
    fn current_state(&self) -> &S {
        &self.current_state
    }
    fn current_state_mut(&mut self) -> &mut S {
        &mut self.current_state
    }
    fn run(&mut self) { /* Placeholder */
    }
    fn advance_state(&mut self) { /* Placeholder */
    }
}

impl<S: State> StateMachine<S> for ExtractSuperState<S> {}

impl<S: State> SMSuperState<S> for ExtractSuperState<S> {}

impl<S: State> SuperState<S> for ExtractSuperState<S> {}

impl ExtractSuperState<ValidateCikFormat> {
    #[must_use]
    pub fn new(input: impl Into<String>) -> Self {
        let input: String = input.into();
        let input_data = ValidateCikFormatInput::new(input.clone());
        let context = ValidateCikFormatContext::new(input);

        Self {
            current_state: ValidateCikFormat::new(input_data, context),
            input: ExtractSuperStateData,
            output: None,
            context: ExtractSuperStateContext,
        }
    }
}

impl ExtractSuperState<PrepareSecRequest> {
    #[must_use]
    pub fn new(validated_cik: Cik, user_agent: impl Into<String>) -> Self {
        let input_data = PrepareSecRequestInput::new(validated_cik.clone(), user_agent.into());
        let context = PrepareSecRequestContext::new(validated_cik);

        Self {
            current_state: PrepareSecRequest::new(input_data, context),
            input: ExtractSuperStateData,
            output: None,
            context: ExtractSuperStateContext,
        }
    }
}

impl ExtractSuperState<ExecuteSecRequest> {
    #[must_use]
    pub const fn new(client: SecClient, request: SecRequest, cik: Cik) -> Self {
        let esr_input = ExecuteSecRequestInput::new(client, request);
        let esr_context = ExecuteSecRequestContext::new(cik);

        Self {
            current_state: ExecuteSecRequest::new(esr_input, esr_context),
            input: ExtractSuperStateData,
            output: None,
            context: ExtractSuperStateContext,
        }
    }
}

// --- Streaming ---

impl NonTerminal for ExtractSuperState<ValidateCikFormat> {
    type Current = ValidateCikFormat;
    type Next = PrepareSecRequest;
}

impl NonTerminal for ExtractSuperState<PrepareSecRequest> {
    type Current = PrepareSecRequest;
    type Next = ExecuteSecRequest;
}

/// Terminal state — no [`NonTerminal`] impl, manual [`IntoStateMachineStream`].
impl IntoStateMachineStream for ExtractSuperState<ExecuteSecRequest> {
    fn into_stream(self, execution_id: uuid::Uuid) -> StateMachineStream {
        Box::pin(async_stream::stream! {
            use crate::traits::state_machine::stream::{StreamEvent, StreamItem, StreamError};

            let mut sm = self;
            let state_name = sm.current_state().state_name().to_string();
            let state_start = std::time::Instant::now();

            // StateStarted
            yield Ok(StreamItem {
                event: StreamEvent::StateStarted,
                state_name: state_name.clone(),
                data: serde_json::to_value(sm.current_state()).unwrap_or_else(|e| {
                    serde_json::json!({ "serialization_error": e.to_string() })
                }),
                event_duration: std::time::Duration::ZERO,
            });

            // Compute
            match sm.current_state_mut().compute_output_data_async().await {
                Ok(()) => {
                    let data = serde_json::to_value(sm.current_state()).unwrap_or_else(|e| {
                        serde_json::json!({ "serialization_error": e.to_string() })
                    });
                    yield Ok(StreamItem {
                        event: StreamEvent::StateCompleted,
                        state_name,
                        data,
                        event_duration: state_start.elapsed(),
                    });
                }
                Err(e) => {
                    #[allow(clippy::useless_conversion)]
                    let state_err: crate::error::State = e.into();
                    let sm_error: crate::error::StateMachine = state_err.into();
                    let data = serde_json::to_value(sm.current_state()).unwrap_or_else(|e| {
                        serde_json::json!({ "serialization_error": e.to_string() })
                    });
                    yield Err(StreamError {
                        event: StreamEvent::StateFailed,
                        execution_id,
                        state_name,
                        data,
                        source: sm_error,
                    });
                }
            }
        })
    }
}

impl Transition<ValidateCikFormat, PrepareSecRequest> for ExtractSuperState<ValidateCikFormat> {
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let next_state = PrepareSecRequest::try_from(self.current_state)?;

        Ok(ExtractSuperState::<PrepareSecRequest> {
            current_state: next_state,
            input: ExtractSuperStateData,
            output: None,
            context: ExtractSuperStateContext,
        })
    }
}

impl Transition<PrepareSecRequest, ExecuteSecRequest> for ExtractSuperState<PrepareSecRequest> {
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let next_state = ExecuteSecRequest::try_from(self.current_state)?;

        Ok(ExtractSuperState::<ExecuteSecRequest> {
            current_state: next_state,
            input: ExtractSuperStateData,
            output: None,
            context: ExtractSuperStateContext,
        })
    }
}

impl SMTransition<PrepareSecRequest, ExecuteSecRequest> for ExtractSuperState<PrepareSecRequest> {
    type NewStateMachine = ExtractSuperState<ExecuteSecRequest>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        // Placeholder implementation - use transition_to_next_state_sec() for actual functionality
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

impl SMTransition<ValidateCikFormat, PrepareSecRequest> for ExtractSuperState<ValidateCikFormat> {
    type NewStateMachine = ExtractSuperState<PrepareSecRequest>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        // Placeholder implementation - use transition_to_next_state_sec() for actual functionality
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::user_agent::constants::DEFAULT_SEC_USER_AGENT;

    #[test]
    fn should_return_super_state_name_with_current_state_when_in_validate_cik_format_state() {
        let input_cik = "1234567890";
        let super_state = ExtractSuperState::<ValidateCikFormat>::new(input_cik);

        let expected_result = "Extract SuperState (Current: Validate CIK Format)";

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_super_state_name_with_current_state_when_in_prepare_sec_request_state() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should be valid");
        let user_agent = DEFAULT_SEC_USER_AGENT.to_string();
        let super_state = ExtractSuperState::<PrepareSecRequest>::new(cik, user_agent);

        let expected_result = "Extract SuperState (Current: Prepare SEC Request)";

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_access_current_validate_cik_format_state_from_super_state() {
        let input_cik = "1234567890";
        let super_state = ExtractSuperState::<ValidateCikFormat>::new(input_cik);

        let expected_result = "Validate CIK Format";

        let result = super_state.current_state().state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_access_current_prepare_sec_request_state_from_super_state() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should be valid");
        let user_agent = DEFAULT_SEC_USER_AGENT.to_string();
        let super_state = ExtractSuperState::<PrepareSecRequest>::new(cik, user_agent);

        let expected_result = "Prepare SEC Request";

        let result = super_state.current_state().state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_transition_from_prepare_sec_request_to_execute_sec_request_state() {
        let input_cik = "1234567890";
        let mut super_state = ExtractSuperState::<ValidateCikFormat>::new(input_cik);

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
        let user_agent = DEFAULT_SEC_USER_AGENT.to_string();
        let super_state = ExtractSuperState::<PrepareSecRequest>::new(cik, user_agent);

        let expected_result = true;
        let result = super_state.transition_to_next_state_sec().is_err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_delegate_computation_to_current_state_when_computing_output_data() {
        let input_cik = "1234567890";
        let mut super_state = ExtractSuperState::<ValidateCikFormat>::new(input_cik);

        let expected_result = Ok(());

        let result = super_state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_transition_from_validate_cik_format_to_prepare_sec_request_state() {
        let input_cik = "1234567890";
        let mut super_state = ExtractSuperState::<ValidateCikFormat>::new(input_cik);

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
    #[should_panic]
    async fn should_fail_transition_when_output_data_not_yet_computed() {
        let input_cik = "1234567890";
        let super_state = ExtractSuperState::<ValidateCikFormat>::new(input_cik);

        let _result = super_state
            .transition_to_next_state_sec()
            .expect("Transition should fail when output data is not yet computed");
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_validate_cik_format_super_state() {
        implements_auto_traits::<ExtractSuperState<ValidateCikFormat>>();
    }

    #[test]
    const fn should_implement_auto_traits_for_prepare_sec_request_super_state() {
        implements_auto_traits::<ExtractSuperState<PrepareSecRequest>>();
    }

    #[test]
    const fn should_implement_auto_traits_for_execute_sec_request_super_state() {
        implements_auto_traits::<ExtractSuperState<ExecuteSecRequest>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_be_thread_safe_for_validate_cik_format_super_state() {
        implements_send::<ExtractSuperState<ValidateCikFormat>>();
        implements_sync::<ExtractSuperState<ValidateCikFormat>>();
    }

    #[test]
    const fn should_be_thread_safe_for_prepare_sec_request_super_state() {
        implements_send::<ExtractSuperState<PrepareSecRequest>>();
        implements_sync::<ExtractSuperState<PrepareSecRequest>>();
    }

    #[test]
    const fn should_be_thread_safe_for_execute_sec_request_super_state() {
        implements_send::<ExtractSuperState<ExecuteSecRequest>>();
        implements_sync::<ExtractSuperState<ExecuteSecRequest>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_validate_cik_format_super_state() {
        implements_debug::<ExtractSuperState<ValidateCikFormat>>();
    }

    #[test]
    const fn should_implement_debug_for_prepare_sec_request_super_state() {
        implements_debug::<ExtractSuperState<PrepareSecRequest>>();
    }

    #[test]
    const fn should_implement_debug_for_execute_sec_request_super_state() {
        implements_debug::<ExtractSuperState<ExecuteSecRequest>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_validate_cik_format_super_state() {
        implements_clone::<ExtractSuperState<ValidateCikFormat>>();
    }

    #[test]
    const fn should_implement_clone_for_prepare_sec_request_super_state() {
        implements_clone::<ExtractSuperState<PrepareSecRequest>>();
    }

    #[test]
    const fn should_implement_clone_for_execute_sec_request_super_state() {
        implements_clone::<ExtractSuperState<ExecuteSecRequest>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_validate_cik_format_super_state() {
        implements_partial_eq::<ExtractSuperState<ValidateCikFormat>>();
    }

    #[test]
    const fn should_implement_partial_eq_for_prepare_sec_request_super_state() {
        implements_partial_eq::<ExtractSuperState<PrepareSecRequest>>();
    }

    #[test]
    const fn should_implement_partial_eq_for_execute_sec_request_super_state() {
        implements_partial_eq::<ExtractSuperState<ExecuteSecRequest>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_validate_cik_format_super_state() {
        implements_hash::<ExtractSuperState<ValidateCikFormat>>();
    }

    #[test]
    const fn should_implement_hash_for_prepare_sec_request_super_state() {
        implements_hash::<ExtractSuperState<PrepareSecRequest>>();
    }

    #[test]
    const fn should_implement_hash_for_execute_sec_request_super_state() {
        implements_hash::<ExtractSuperState<ExecuteSecRequest>>();
    }
}
