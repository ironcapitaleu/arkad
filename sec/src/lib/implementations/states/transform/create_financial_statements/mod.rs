//! # Create Financial Statements State
//!
//! This module provides the [`CreateFinancialStatements`] state and related types for creating
//! financial statements from parsed company data as part of the SEC transform workflow.
//!
//! ## Overview
//! The [`CreateFinancialStatements`] state is responsible for transforming [`CompanyData`](crate::shared::financial::company_data::CompanyData)
//! into structured financial statements. This is currently a **scaffold** -- the compute logic
//! is a stub that will be filled in a future iteration.
//!
//! ## Components
//! - [`context`]: Defines the context and updater types for the financial statement creation process.
//! - [`data`]: Contains input and output data structures for the state, including updaters and builders.
//! - [`CreateFinancialStatementsContext`]: Context data type for the state.
//! - [`CreateFinancialStatementsInput`]: Input data type holding the [`CompanyData`](crate::shared::financial::company_data::CompanyData).
//! - [`CreateFinancialStatementsOutput`]: Placeholder output data type.
//!
//! ## Usage
//! This state is intended to be used in the transform phase of the SEC state machine ETL pipeline,
//! after company facts have been parsed. It is designed to be composed with other states for robust
//! and testable SEC filings processing workflows.
//!
//! ## Example
//! ```rust
//! use std::collections::HashMap;
//! use tokio;
//!
//! use sec::implementations::states::transform::create_financial_statements::*;
//! use sec::shared::cik::Cik;
//! use sec::shared::financial::company_data::CompanyData;
//! use sec::shared::financial::entity_name::EntityName;
//! use sec::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let company_data = CompanyData::new(
//!         Cik::new("0000320193").unwrap(),
//!         EntityName::new("Apple Inc."),
//!         HashMap::new(),
//!     );
//!     let input = CreateFinancialStatementsInput::new(company_data);
//!     let context = CreateFinancialStatementsContext::default();
//!
//!     let mut state = CreateFinancialStatements::new(input, context);
//!     state.compute_output_data_async().await.unwrap();
//!     let output = state.output_data().unwrap();
//!     assert_eq!(output, &CreateFinancialStatementsOutput);
//! }
//! ```
//!
//! ## See Also
//! - [`crate::implementations::states::transform`]: Parent module for transform-related states.
//! - [`crate::shared::financial::company_data::CompanyData`]: Core data type used as input.
//! - [`crate::traits::state_machine::state::State`]: State trait implemented by [`CreateFinancialStatements`].
//!
//! ## Testing
//! This module includes comprehensive unit tests covering state behavior, trait compliance, and stub output.

use std::fmt;
use std::hash::{Hash, Hasher};

use async_trait::async_trait;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::CreateFinancialStatementsContext;
pub use data::CreateFinancialStatementsInput;
pub use data::CreateFinancialStatementsOutput;

// Deviation: `CreateFinancialStatementsInput` wraps `CompanyData` which uses
// manual `Hash`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` implementations due to
// containing a `HashMap`. This struct therefore also uses manual implementations
// for these traits.
#[derive(Debug, Clone, serde::Serialize)]
/// State that creates financial statements from parsed company data.
///
/// This state takes [`CompanyData`](crate::shared::financial::company_data::CompanyData) as input and produces
/// structured financial statements as output. Currently this is a **scaffold** -- the
/// compute logic is a stub that returns a placeholder output.
///
/// # Behavior
/// - Accepts [`CompanyData`](crate::shared::financial::company_data::CompanyData) containing resolved financial facts.
/// - Produces a placeholder [`CreateFinancialStatementsOutput`].
/// - The actual financial statement creation logic will be implemented in a future iteration.
///
/// # Output
/// The output is stored internally after calling [`State::compute_output_data_async`].
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use sec::implementations::states::transform::create_financial_statements::*;
/// use sec::shared::cik::Cik;
/// use sec::shared::financial::company_data::CompanyData;
/// use sec::shared::financial::entity_name::EntityName;
///
/// let company_data = CompanyData::new(
///     Cik::new("0000320193").unwrap(),
///     EntityName::new("Apple Inc."),
///     HashMap::new(),
/// );
/// let input = CreateFinancialStatementsInput::new(company_data);
/// let context = CreateFinancialStatementsContext::default();
/// let state = CreateFinancialStatements::new(input, context);
/// ```
pub struct CreateFinancialStatements {
    input: CreateFinancialStatementsInput,
    context: CreateFinancialStatementsContext,
    output: Option<CreateFinancialStatementsOutput>,
}

impl CreateFinancialStatements {
    /// Creates a new [`CreateFinancialStatements`] state with the given input and context.
    #[must_use]
    pub const fn new(
        input: CreateFinancialStatementsInput,
        context: CreateFinancialStatementsContext,
    ) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its components. Used for state transitions.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        CreateFinancialStatementsInput,
        Option<CreateFinancialStatementsOutput>,
        CreateFinancialStatementsContext,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl State for CreateFinancialStatements {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        // Stub: actual financial statement creation logic will be implemented in a future iteration.
        self.output = Some(CreateFinancialStatementsOutput);
        Ok(())
    }
}

impl SMState for CreateFinancialStatements {
    type InputData = CreateFinancialStatementsInput;
    type OutputData = CreateFinancialStatementsOutput;
    type Context = CreateFinancialStatementsContext;

    fn state_name(&self) -> impl ToString {
        STATE_NAME
    }

    /// Blocking wrapper around [`compute_output_data_async`](crate::traits::state_machine::state::State::compute_output_data_async).
    ///
    /// Detects whether a tokio runtime is available and runs the async computation
    /// synchronously. This allows SEC states to be used as regular `SMState` implementations.
    ///
    /// # Panics
    /// Panics if the async computation returns an error.
    fn compute_output_data(&mut self) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(self.compute_output_data_async()))
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.compute_output_data_async())
        };

        if let Err(e) = result {
            let state_err: crate::error::State = e;
            panic!("compute_output_data failed: {state_err}")
        }
    }

    fn context_data(&self) -> &Self::Context {
        &self.context
    }

    fn input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl Default for CreateFinancialStatements {
    /// Returns a default state with default input and context.
    fn default() -> Self {
        Self::new(
            CreateFinancialStatementsInput::default(),
            CreateFinancialStatementsContext::default(),
        )
    }
}

impl PartialEq for CreateFinancialStatements {
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input && self.context == other.context && self.output == other.output
    }
}

impl Eq for CreateFinancialStatements {}

// Deviation: delegates to `CreateFinancialStatementsInput`'s manual `Hash` implementation.
impl Hash for CreateFinancialStatements {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.input.hash(state);
        self.context.hash(state);
        self.output.hash(state);
    }
}

impl PartialOrd for CreateFinancialStatements {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Deviation: delegates to `CreateFinancialStatementsInput`'s manual `Ord` implementation.
impl Ord for CreateFinancialStatements {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.input
            .cmp(&other.input)
            .then_with(|| self.context.cmp(&other.context))
    }
}

impl fmt::Display for CreateFinancialStatements {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};
    use tokio;

    #[test]
    fn should_return_name_of_state_when_in_create_financial_statements_state() {
        let state = CreateFinancialStatements::default();

        let expected_result = String::from("Create Financial Statements");

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_input_data_when_in_initial_state() {
        let state = CreateFinancialStatements::default();

        let expected_result = &CreateFinancialStatementsInput::default();

        let result = state.input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let state = CreateFinancialStatements::default();

        let _result = state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let state = CreateFinancialStatements::default();

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let state = CreateFinancialStatements::default();

        let expected_result = &CreateFinancialStatementsContext::default();

        let result = state.context_data();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_trait() {
        implements_auto_traits::<CreateFinancialStatements>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_trait() {
        implements_send::<CreateFinancialStatements>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_trait() {
        implements_sync::<CreateFinancialStatements>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_trait() {
        implements_send::<CreateFinancialStatements>();
        implements_sync::<CreateFinancialStatements>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_trait() {
        implements_sized::<CreateFinancialStatements>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_trait() {
        implements_hash::<CreateFinancialStatements>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_trait() {
        implements_partial_eq::<CreateFinancialStatements>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_trait() {
        implements_eq::<CreateFinancialStatements>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_trait() {
        implements_partial_ord::<CreateFinancialStatements>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_trait() {
        implements_ord::<CreateFinancialStatements>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_state_trait() {
        implements_default::<CreateFinancialStatements>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_trait() {
        implements_debug::<CreateFinancialStatements>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_trait() {
        implements_clone::<CreateFinancialStatements>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_trait() {
        implements_unpin::<CreateFinancialStatements>();
    }

    #[test]
    fn should_return_default_context_data_when_called_with_state_reference() {
        let state = &CreateFinancialStatements::default();
        let ref_to_state = &CreateFinancialStatements::default();

        let expected_result = state.context_data();

        let result = ref_to_state.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_state = &mut CreateFinancialStatements::default();

        let expected_result = false;

        let result = ref_to_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(
        expected = "State with valid input should always produce output after computation"
    )]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_reference_state()
     {
        let ref_to_state = &CreateFinancialStatements::default();

        let _result = ref_to_state
            .output_data()
            .expect("State with valid input should always produce output after computation");
    }

    #[test]
    fn should_return_name_of_state_when_calling_reference_to_state() {
        let ref_to_state = &CreateFinancialStatements::default();

        let expected_result = String::from("Create Financial Statements");

        let result = ref_to_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_input_data_when_reference_state_in_initial_state() {
        let ref_to_state = &CreateFinancialStatements::default();

        let expected_result = &CreateFinancialStatementsInput::default();

        let result = ref_to_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_change_input_data_when_computing_output_data() {
        let mut state = CreateFinancialStatements::default();

        let expected_result = &state.input_data().clone();

        state
            .compute_output_data_async()
            .await
            .expect("Default test state should always compute output successfully");
        let result = state.input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_placeholder_output_when_computing_output_data() {
        let mut state = CreateFinancialStatements::default();

        let expected_result = &CreateFinancialStatementsOutput;

        state
            .compute_output_data_async()
            .await
            .expect("Default test state should always compute output successfully");
        let result = state.output_data().unwrap();

        assert_eq!(result, expected_result);
    }
}
