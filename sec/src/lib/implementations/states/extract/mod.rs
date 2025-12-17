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
pub mod validate_sec_response;

use std::fmt::Display;

use crate::error::State as StateError;
use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::{
    ExecuteSecRequest, ExecuteSecRequestContext, ExecuteSecRequestInputData,
    ExecuteSecRequestOutputData,
};
use crate::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInputData,
    PrepareSecRequestOutputData,
};
use crate::implementations::states::extract::validate_cik_format::{
    ValidateCikFormat, ValidateCikFormatContext, ValidateCikFormatInput,
    ValidateCikFormatOutput,
};

use crate::implementations::states::extract::validate_sec_response::{
    ValidateSecResponse, ValidateSecResponseContext, ValidateSecResponseInputData,
};

use crate::shared::cik::Cik;
use crate::shared::sec_client::SecClient;
use crate::shared::sec_request::SecRequest;
use crate::shared::user_agent::constants::DEFAULT_SEC_USER_AGENT;

use async_trait::async_trait;

use crate::prelude::*;
use state_maschine::prelude::{StateMachine as SMStateMachine, Transition as SMTransition};

/// Data structure for the Extract super-state.
///
/// Currently serves as a placeholder type with unit update semantics for the [`ExtractSuperState`].
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractSuperStateData;

impl SMStateData for ExtractSuperStateData {
    type UpdateType = ();
    fn get_state(&self) -> &Self {
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
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractSuperStateContext;

impl SMContextData for ExtractSuperStateContext {
    type UpdateType = ();
    fn get_context(&self) -> &Self {
        self
    }
    fn update_context(&mut self, (): Self::UpdateType) {}
}

impl ContextData for ExtractSuperStateContext {
    fn get_max_retries(&self) -> u32 {
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
            self.current_state.get_state_name().to_string()
        )
    }
}

impl<S: State> SMState for ExtractSuperState<S> {
    type InputData = ExtractSuperStateData;
    type OutputData = ExtractSuperStateData;
    type Context = ExtractSuperStateContext;

    fn get_state_name(&self) -> impl ToString {
        format!(
            "Extract SuperState (Current: {})",
            self.current_state.get_state_name().to_string()
        )
    }
    fn get_input_data(&self) -> &Self::InputData {
        &self.input
    }
    fn compute_output_data(&mut self) { /* handled by async version */
    }
    fn get_output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
    fn get_context_data(&self) -> &Self::Context {
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
    fn get_current_state(&self) -> &S {
        &self.current_state
    }
    fn get_current_state_mut(&mut self) -> &mut S {
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

impl From<ValidateCikFormatOutput> for PrepareSecRequestContext {
    fn from(output_data: ValidateCikFormatOutput) -> Self {
        Self::new(output_data.validated_cik)
    }
}

impl From<ValidateCikFormatOutput> for PrepareSecRequestInputData {
    fn from(output_data: ValidateCikFormatOutput) -> Self {
        Self::new(
            output_data.validated_cik,
            DEFAULT_SEC_USER_AGENT.to_string(),
        )
    }
}

impl TryFrom<ValidateCikFormat> for PrepareSecRequest {
    type Error = TransitionError;

    fn try_from(state: ValidateCikFormat) -> Result<Self, TransitionError> {
        let output_data = match state.get_output_data() {
            Some(data) => data.clone(),
            None => {
                return Err(transition::MissingOutputData::new(
                    "Extract SuperState",
                    state.get_state_name().to_string(),
                )
                .into());
            }
        };

        let new_context: PrepareSecRequestContext = output_data.clone().into();
        let new_input: PrepareSecRequestInputData = output_data.into();

        Ok(Self::new(new_input, new_context))
    }
}

impl From<PrepareSecRequestContext> for ExecuteSecRequestContext {
    fn from(context: PrepareSecRequestContext) -> Self {
        Self::new(context.cik)
    }
}

impl From<ExecuteSecRequestContext> for ValidateSecResponseContext {
    fn from(context: ExecuteSecRequestContext) -> Self {
        Self::new(context.cik)
    }
}

impl From<PrepareSecRequestOutputData> for ExecuteSecRequestInputData {
    fn from(output_data: PrepareSecRequestOutputData) -> Self {
        Self::new(output_data.client, output_data.request)
    }
}

impl From<ExecuteSecRequestOutputData> for ValidateSecResponseInputData {
    fn from(output_data: ExecuteSecRequestOutputData) -> Self {
        Self::new(output_data.response)
    }
}

impl TryFrom<PrepareSecRequest> for ExecuteSecRequest {
    type Error = TransitionError;

    fn try_from(state: PrepareSecRequest) -> Result<Self, TransitionError> {
        let output_data = match state.get_output_data() {
            Some(data) => data.clone(),
            None => {
                return Err(transition::MissingOutputData::new(
                    "Extract SuperState",
                    state.get_state_name().to_string(),
                )
                .into());
            }
        };

        let state_context = state.get_context_data().clone();
        let new_context: ExecuteSecRequestContext = state_context.into();
        let new_input: ExecuteSecRequestInputData = output_data.into();

        Ok(Self::new(new_input, new_context))
    }
}

impl TryFrom<ExecuteSecRequest> for ValidateSecResponse {
    type Error = TransitionError;

    fn try_from(state: ExecuteSecRequest) -> Result<Self, TransitionError> {
        let output_data = match state.get_output_data() {
            Some(data) => data.clone(),
            None => {
                return Err(transition::MissingOutputData::new(
                    "Extract SuperState",
                    state.get_state_name().to_string(),
                )
                .into());
            }
        };

        let state_context = state.get_context_data().clone();
        let new_context: ValidateSecResponseContext = state_context.into();
        let new_input: ValidateSecResponseInputData = output_data.into();

        Ok(Self::new(new_input, new_context))
    }
}

impl ExtractSuperState<ValidateCikFormat> {
    #[must_use]
    pub fn new(input: &str) -> Self {
        let input_data = ValidateCikFormatInput::new(input);
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
    pub fn new(validated_cik: Cik, user_agent: String) -> Self {
        let psr_input = PrepareSecRequestInputData::new(validated_cik.clone(), user_agent);
        let psr_context = PrepareSecRequestContext::new(validated_cik);

        Self {
            current_state: PrepareSecRequest::new(psr_input, psr_context),
            input: ExtractSuperStateData,
            output: None,
            context: ExtractSuperStateContext,
        }
    }
}

impl ExtractSuperState<ExecuteSecRequest> {
    #[must_use]
    pub const fn new(client: SecClient, request: SecRequest, cik: Cik) -> Self {
        let esr_input = ExecuteSecRequestInputData::new(client, request);
        let esr_context = ExecuteSecRequestContext::new(cik);

        Self {
            current_state: ExecuteSecRequest::new(esr_input, esr_context),
            input: ExtractSuperStateData,
            output: None,
            context: ExtractSuperStateContext,
        }
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

impl Transition<ExecuteSecRequest, ValidateSecResponse> for ExtractSuperState<ExecuteSecRequest> {
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let next_state = ValidateSecResponse::try_from(self.current_state)?;

        Ok(ExtractSuperState::<ValidateSecResponse> {
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

impl SMTransition<ExecuteSecRequest, ValidateSecResponse> for ExtractSuperState<ExecuteSecRequest> {
    type NewStateMachine = ExtractSuperState<ValidateSecResponse>;

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
    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::user_agent::constants::DEFAULT_SEC_USER_AGENT;
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};
    use tokio;

    #[test]
    fn should_return_super_state_name_with_current_state_when_in_validate_cik_format_state() {
        let input_cik = "1234567890";
        let super_state = ExtractSuperState::<ValidateCikFormat>::new(input_cik);

        let expected_result = "Extract SuperState (Current: CIK Format Validation)";

        let result = super_state.get_state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_super_state_name_with_current_state_when_in_prepare_sec_request_state() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should be valid");
        let user_agent = DEFAULT_SEC_USER_AGENT.to_string();
        let super_state = ExtractSuperState::<PrepareSecRequest>::new(cik, user_agent);

        let expected_result = "Extract SuperState (Current: Prepare SEC Request)";

        let result = super_state.get_state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_access_current_validate_cik_format_state_from_super_state() {
        let input_cik = "1234567890";
        let super_state = ExtractSuperState::<ValidateCikFormat>::new(input_cik);

        let expected_state_name = "CIK Format Validation";

        let result = super_state.get_current_state().get_state_name().to_string();

        assert_eq!(result, expected_state_name);
    }

    #[test]
    fn should_access_current_prepare_sec_request_state_from_super_state() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should be valid");
        let user_agent = DEFAULT_SEC_USER_AGENT.to_string();
        let super_state = ExtractSuperState::<PrepareSecRequest>::new(cik, user_agent);

        let expected_state_name = "Prepare SEC Request";

        let result = super_state.get_current_state().get_state_name().to_string();

        assert_eq!(result, expected_state_name);
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

        let result = state.get_state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_transition_from_execute_sec_request_to_validate_sec_response_state() {
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

        let mut super_state = super_state
            .transition_to_next_state_sec()
            .expect("Should transition to ExecuteSecRequest");

        super_state
            .compute_output_data_async()
            .await
            .expect("Should compute output data");

        let expected_result = "Extract SuperState (Current: Validate SEC Response)";

        let state = super_state
            .transition_to_next_state_sec()
            .expect("Should transition to ValidateSecResponse");

        let result = state.get_state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_transition_from_execute_sec_request_when_output_data_not_yet_computed() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should be valid");
        let client = SecClient::new("Test Company contact@test.com").expect("Valid user agent");
        let request = SecRequest::new(&cik);
        let super_state = ExtractSuperState::<ExecuteSecRequest>::new(client, request, cik);

        let result = super_state.transition_to_next_state_sec();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_fail_transition_from_prepare_sec_request_when_output_data_not_yet_computed() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should be valid");
        let user_agent = DEFAULT_SEC_USER_AGENT.to_string();
        let super_state = ExtractSuperState::<PrepareSecRequest>::new(cik, user_agent);

        let result = super_state.transition_to_next_state_sec();

        assert!(result.is_err());
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

        let expected_result_type = "Extract SuperState (Current: Prepare SEC Request)";

        let result = super_state.transition_to_next_state_sec().unwrap();

        assert_eq!(result.get_state_name().to_string(), expected_result_type);
    }

    #[tokio::test]
    async fn should_fail_transition_when_output_data_not_yet_computed() {
        let input_cik = "1234567890";
        let super_state = ExtractSuperState::<ValidateCikFormat>::new(input_cik);

        let result = super_state.transition_to_next_state_sec();

        assert!(result.is_err());
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

    #[test]
    const fn should_implement_auto_traits_for_validate_sec_response_super_state() {
        implements_auto_traits::<ExtractSuperState<ValidateSecResponse>>();
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

    #[test]
    const fn should_be_thread_safe_for_validate_sec_response_super_state() {
        implements_send::<ExtractSuperState<ValidateSecResponse>>();
        implements_sync::<ExtractSuperState<ValidateSecResponse>>();
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

    #[test]
    const fn should_implement_debug_for_validate_sec_response_super_state() {
        implements_debug::<ExtractSuperState<ValidateSecResponse>>();
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

    #[test]
    const fn should_implement_clone_for_validate_sec_response_super_state() {
        implements_clone::<ExtractSuperState<ValidateSecResponse>>();
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

    #[test]
    const fn should_implement_partial_eq_for_validate_sec_response_super_state() {
        implements_partial_eq::<ExtractSuperState<ValidateSecResponse>>();
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

    #[test]
    const fn should_implement_hash_for_validate_sec_response_super_state() {
        implements_hash::<ExtractSuperState<ValidateSecResponse>>();
    }
}
