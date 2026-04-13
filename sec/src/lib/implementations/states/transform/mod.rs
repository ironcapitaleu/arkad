//! # Transform State Module
//!
//! This module provides state implementations for the transform phase of the SEC filings ETL workflow.
//! It handles conversion of extracted data into structured, normalized, and enriched forms for downstream processing.
//!
//! ## Components
//! - [`create_financial_statements`]: Creates structured financial statements from parsed company data.
//! - [`parse_company_facts`]: Parses SEC Company Facts JSON into structured [`CompanyData`].
//! - [`TransformSuperState`]: Super-state that orchestrates the transform workflow and state transitions.
//!
//! ## State Flow
//! The transformation follows this progression: [`ParseCompanyFacts`] → [`CreateFinancialStatements`]
//!
//! ## Example
//! ```rust
//! use std::collections::HashMap;
//! use sec::implementations::states::transform::*;
//! use sec::implementations::states::transform::parse_company_facts::ParseCompanyFacts;
//! use sec::shared::cik::Cik;
//! use sec::shared::financial::company_data::CompanyData;
//! use sec::shared::financial::entity_name::EntityName;
//! use sec::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let response = serde_json::json!({});
//!     let cik = Cik::new("0000320193")?;
//!     let mut transform_state = TransformSuperState::<ParseCompanyFacts>::new(response, cik);
//!     Ok(())
//! }
//! ```

pub mod create_financial_statements;
pub mod parse_company_facts;

use std::fmt::Display;

use async_trait::async_trait;

use crate::error::State as StateError;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::transform::create_financial_statements::{
    CreateFinancialStatements, CreateFinancialStatementsContext, CreateFinancialStatementsInput,
};
use crate::implementations::states::transform::parse_company_facts::{
    ParseCompanyFacts, ParseCompanyFactsContext, ParseCompanyFactsInput,
};
use crate::prelude::*;
use crate::shared::cik::Cik;
use crate::shared::financial::company_data::CompanyData;
use state_maschine::prelude::{StateMachine as SMStateMachine, Transition as SMTransition};

/// Data structure for the Transform super-state.
///
/// Currently serves as a placeholder type with unit update semantics for the [`TransformSuperState`].
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct TransformSuperStateData;

impl SMStateData for TransformSuperStateData {
    type UpdateType = ();
    fn state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, (): Self::UpdateType) {}
}

impl StateData for TransformSuperStateData {
    fn update_state(&mut self, (): Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

/// Context data structure for the Transform super-state.
///
/// Provides configuration and runtime context for the [`TransformSuperState`], including retry policies.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct TransformSuperStateContext;

impl SMContext for TransformSuperStateContext {
    type UpdateType = ();
    fn context(&self) -> &Self {
        self
    }
    fn update_context(&mut self, (): Self::UpdateType) {}
}

impl Context for TransformSuperStateContext {
    fn max_retries(&self) -> u32 {
        0
    }
}

/// A hierarchical super-state that orchestrates the transform phase of the SEC ETL pipeline.
///
/// Manages progression through transform states like [`ParseCompanyFacts`] and [`CreateFinancialStatements`],
/// providing type-safe transitions and unified state machine interfaces.
///
/// # Type Parameter
/// - `S`: The current active state, which must implement the [`State`] trait
///
/// # State Transitions
/// Supports transitions: `ParseCompanyFacts` → `CreateFinancialStatements`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct TransformSuperState<S: State> {
    current_state: S,
    input: TransformSuperStateData,
    output: Option<TransformSuperStateData>,
    context: TransformSuperStateContext,
}

impl<S: State> Display for TransformSuperState<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transform SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }
}

impl<S: State> SMState for TransformSuperState<S> {
    type InputData = TransformSuperStateData;
    type OutputData = TransformSuperStateData;
    type Context = TransformSuperStateContext;

    fn state_name(&self) -> impl ToString {
        format!(
            "Transform SuperState (Current: {})",
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
impl<S: State> State for TransformSuperState<S> {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.current_state
            .compute_output_data_async()
            .await
            .map_err(std::convert::Into::into)
    }
}

impl<S: State> SMStateMachine<S> for TransformSuperState<S> {
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

impl<S: State> StateMachine<S> for TransformSuperState<S> {}

impl<S: State> SMSuperState<S> for TransformSuperState<S> {}

impl<S: State> SuperState<S> for TransformSuperState<S> {}

impl TransformSuperState<ParseCompanyFacts> {
    /// Creates a new [`TransformSuperState`] starting at the [`ParseCompanyFacts`] state.
    ///
    /// # Arguments
    /// * `response` - The raw SEC Company Facts JSON response to parse.
    /// * `cik` - The validated CIK for the company being processed.
    #[must_use]
    pub const fn new(response: serde_json::Value, cik: Cik) -> Self {
        let input_data = ParseCompanyFactsInput::new(response);
        let context = ParseCompanyFactsContext::new(cik);

        Self {
            current_state: ParseCompanyFacts::new(input_data, context),
            input: TransformSuperStateData,
            output: None,
            context: TransformSuperStateContext,
        }
    }
}

impl TransformSuperState<CreateFinancialStatements> {
    /// Creates a new [`TransformSuperState`] starting at the [`CreateFinancialStatements`] state.
    ///
    /// # Arguments
    /// * `company_data` - The parsed company data to transform into financial statements.
    /// * `cik` - The validated CIK for the company being processed.
    #[must_use]
    pub const fn new(company_data: CompanyData, cik: Cik) -> Self {
        let input_data = CreateFinancialStatementsInput::new(company_data);
        let context = CreateFinancialStatementsContext::new(cik);

        Self {
            current_state: CreateFinancialStatements::new(input_data, context),
            input: TransformSuperStateData,
            output: None,
            context: TransformSuperStateContext,
        }
    }
}

// --- Streaming ---

impl NonTerminal for TransformSuperState<ParseCompanyFacts> {
    type Current = ParseCompanyFacts;
    type Next = CreateFinancialStatements;
}

/// Terminal state — no [`NonTerminal`] impl, manual [`IntoStateMachineStream`].
impl IntoStateMachineStream for TransformSuperState<CreateFinancialStatements> {
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

impl Transition<ParseCompanyFacts, CreateFinancialStatements>
    for TransformSuperState<ParseCompanyFacts>
{
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        let next_state = CreateFinancialStatements::try_from(self.current_state)?;

        Ok(TransformSuperState::<CreateFinancialStatements> {
            current_state: next_state,
            input: TransformSuperStateData,
            output: None,
            context: TransformSuperStateContext,
        })
    }
}

impl SMTransition<ParseCompanyFacts, CreateFinancialStatements>
    for TransformSuperState<ParseCompanyFacts>
{
    type NewStateMachine = TransformSuperState<CreateFinancialStatements>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        // Placeholder implementation - use transition_to_next_state_sec() for actual functionality
        Err(
            "Use transition_to_next_state_sec() for SEC-specific transitions with rich error handling",
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::financial::company_data::CompanyData;
    use crate::shared::financial::entity_name::EntityName;

    fn test_cik() -> Cik {
        Cik::new("0000320193").expect("Hardcoded CIK should always be valid")
    }

    fn test_company_data() -> CompanyData {
        CompanyData::new(test_cik(), EntityName::new("Apple Inc."), HashMap::new())
    }

    #[test]
    fn should_return_super_state_name_with_current_state_when_in_parse_company_facts_state() {
        let response = serde_json::json!({});
        let super_state = TransformSuperState::<ParseCompanyFacts>::new(response, test_cik());

        let expected_result = "Transform SuperState (Current: Parse Company Facts)";

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_super_state_name_with_current_state_when_in_create_financial_statements_state()
    {
        let super_state =
            TransformSuperState::<CreateFinancialStatements>::new(test_company_data(), test_cik());

        let expected_result = "Transform SuperState (Current: Create Financial Statements)";

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_access_current_parse_company_facts_state_from_super_state() {
        let response = serde_json::json!({});
        let super_state = TransformSuperState::<ParseCompanyFacts>::new(response, test_cik());

        let expected_result = "Parse Company Facts";

        let result = super_state.current_state().state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_access_current_create_financial_statements_state_from_super_state() {
        let super_state =
            TransformSuperState::<CreateFinancialStatements>::new(test_company_data(), test_cik());

        let expected_result = "Create Financial Statements";

        let result = super_state.current_state().state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_delegate_computation_to_current_state_when_computing_output_data() {
        let super_state =
            TransformSuperState::<CreateFinancialStatements>::new(test_company_data(), test_cik());
        let mut super_state = super_state;

        let expected_result = Ok(());

        let result = super_state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_fail_transition_from_parse_company_facts_when_output_data_not_yet_computed() {
        let response = serde_json::json!({});
        let super_state = TransformSuperState::<ParseCompanyFacts>::new(response, test_cik());

        let expected_result = true;
        let result = super_state.transition_to_next_state_sec().is_err();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    #[should_panic(expected = "Transition should fail when output data is not yet computed")]
    async fn should_fail_transition_when_output_data_not_yet_computed() {
        let response = serde_json::json!({});
        let super_state = TransformSuperState::<ParseCompanyFacts>::new(response, test_cik());

        let _result = super_state
            .transition_to_next_state_sec()
            .expect("Transition should fail when output data is not yet computed");
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_parse_company_facts_super_state() {
        implements_auto_traits::<TransformSuperState<ParseCompanyFacts>>();
    }

    #[test]
    const fn should_implement_auto_traits_for_create_financial_statements_super_state() {
        implements_auto_traits::<TransformSuperState<CreateFinancialStatements>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_be_thread_safe_for_parse_company_facts_super_state() {
        implements_send::<TransformSuperState<ParseCompanyFacts>>();
        implements_sync::<TransformSuperState<ParseCompanyFacts>>();
    }

    #[test]
    const fn should_be_thread_safe_for_create_financial_statements_super_state() {
        implements_send::<TransformSuperState<CreateFinancialStatements>>();
        implements_sync::<TransformSuperState<CreateFinancialStatements>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_for_parse_company_facts_super_state() {
        implements_debug::<TransformSuperState<ParseCompanyFacts>>();
    }

    #[test]
    const fn should_implement_debug_for_create_financial_statements_super_state() {
        implements_debug::<TransformSuperState<CreateFinancialStatements>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_for_parse_company_facts_super_state() {
        implements_clone::<TransformSuperState<ParseCompanyFacts>>();
    }

    #[test]
    const fn should_implement_clone_for_create_financial_statements_super_state() {
        implements_clone::<TransformSuperState<CreateFinancialStatements>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_for_parse_company_facts_super_state() {
        implements_partial_eq::<TransformSuperState<ParseCompanyFacts>>();
    }

    #[test]
    const fn should_implement_partial_eq_for_create_financial_statements_super_state() {
        implements_partial_eq::<TransformSuperState<CreateFinancialStatements>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_for_parse_company_facts_super_state() {
        implements_hash::<TransformSuperState<ParseCompanyFacts>>();
    }

    #[test]
    const fn should_implement_hash_for_create_financial_statements_super_state() {
        implements_hash::<TransformSuperState<CreateFinancialStatements>>();
    }
}
