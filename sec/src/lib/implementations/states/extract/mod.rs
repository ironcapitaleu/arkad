//! # Extract State Module
//!
//! This module contains state implementations responsible for the extraction and initial validation of raw SEC filings data.
//! It provides the entry point for the Extract phase in the SEC state machine ETL workflow.
//!
//! ## Submodules
//! - [`validate_cik_format`]: Implements states for validating and extracting CIK (Central Index Key) information from SEC filings, including format checks and normalization routines.
//!
//! The extract states are designed to be composed within state machines, enabling robust, testable, and extensible data ingestion pipelines for SEC filings processing.
//!
//! See the documentation for each submodule for details on their specific responsibilities and usage.

pub mod validate_cik_format;

use crate::error::State as StateError;
use crate::implementations::states::extract::validate_cik_format::{ValidateCikFormatContext, ValidateCikFormatInputData};
use crate::prelude::*;
use async_trait::async_trait;
use validate_cik_format::ValidateCikFormat;

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

/// The `ExtractSuperState` is a hierarchical state that manages the states of the extraction phase.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtractSuperState<S: State> {
    current_state: S,
    input: ExtractSuperStateData,
    output: Option<ExtractSuperStateData>,
    context: ExtractSuperStateContext,
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

impl ExtractSuperState<ValidateCikFormat> {
    #[must_use]
    pub fn new(
        input: impl ToString,
    ) -> Self {

        let vcf_input = ValidateCikFormatInputData::new(&input);
        let vcf_context = ValidateCikFormatContext::new(&input);

        Self {
            current_state: ValidateCikFormat::new(vcf_input, vcf_context),
            input: ExtractSuperStateData,
            output: None,
            context: ExtractSuperStateContext,
        }
    }
}
