//! # Sample SEC Super State Fixture
//!
//! This module provides the [`SampleSecSuperState`] and related types, which serve as a test fixture
//! for creating super states within the `sec` state machine framework.

use std::fmt;

use async_trait::async_trait;
use state_maschine::prelude::{
    State as SMState, StateMachine as SMStateMachine, SuperState as SMSuperState,
};

use crate::error::State as StateError;
use crate::prelude::*;
use crate::tests::common::sample_sec_state::SampleSecState;

pub mod sample_sec_super_state_context;
pub mod sample_sec_super_state_data;

pub use sample_sec_super_state_context::SampleSecSuperStateContext;
pub use sample_sec_super_state_data::SampleSecSuperStateData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleSecSuperState<S: State> {
    current_state: S,
    input: SampleSecSuperStateData,
    output: Option<SampleSecSuperStateData>,
    context: SampleSecSuperStateContext,
}

impl SampleSecSuperState<SampleSecState> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_state: SampleSecState::default(),
            input: SampleSecSuperStateData::default(),
            output: None,
            context: SampleSecSuperStateContext::default(),
        }
    }
}

impl<S: State> fmt::Display for SampleSecSuperState<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SampleSecSuperState({})",
            self.get_state_name().to_string()
        )
    }
}

#[async_trait]
impl<S: State> State for SampleSecSuperState<S> {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.current_state
            .compute_output_data_async()
            .await
            .map_err(Into::into)
    }
}

impl<S: State> SMState for SampleSecSuperState<S> {
    type InputData = SampleSecSuperStateData;
    type OutputData = SampleSecSuperStateData;
    type Context = SampleSecSuperStateContext;

    fn get_state_name(&self) -> impl ToString {
        format!(
            "Sample SEC SuperState (Current: {})",
            self.current_state.get_state_name().to_string()
        )
    }

    fn compute_output_data(&mut self) {
        // handled by async version
    }

    fn get_context_data(&self) -> &Self::Context {
        &self.context
    }

    fn get_input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn get_output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl<S: State> SMStateMachine<S> for SampleSecSuperState<S> {
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

impl<S: State> StateMachine<S> for SampleSecSuperState<S> {}

impl<S: State> SMSuperState<S> for SampleSecSuperState<S> {}

impl<S: State> SuperState<S> for SampleSecSuperState<S> {}
