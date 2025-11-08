//! # Sample SEC State Fixture
//!
//! This module provides the [`SampleSecState`] and related types, which serve as a test fixture and template
//! for creating new states within the `sec` state machine framework.
//!
//! ## Overview
//! The [`SampleSecState`] is a generic state implementation designed for testing and demonstration purposes.
//! It showcases the required structure for a type implementing the [`State`] trait, including its data and context,
//! but with minimal, "hello world" logic. It is not intended for production use but rather as a blueprint.
//!
//! ## Components
//! - [`sec_context`]: Defines the sample context data (`SampleSecStateContext`) and updater types.
//! - [`sec_data`]: Contains sample input (`SampleSecStateInputData`) and output (`SampleSecStateOutputData`) data structures.
//!
//! ## Usage
//! This state is intended to be used within the test suite to create simple state machines or to verify
//! the behavior of transitions and other framework components. It can also be copied and modified to
//! bootstrap the creation of a new, concrete state.
//!
//! ## Example
//! ```rust
//! use tokio;
//!
//! use sec::tests::common::sample_sec_state::*;
//! use sec::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!
//!     let input = SampleSecStateInputData::default();
//!     let context = SampleSecStateContext::default();
//!
//!     let expected_result = "Hello World!";
//!
//!     let mut sample_state = SampleSecState::new(input, context);
//!     sample_state.compute_output_data_async().await.unwrap();
//!     let sample_output = sample_state.get_output_data().unwrap();
//!     let result = &sample_output.output_data;
//!
//!     assert_eq!(result, expected_result);
//! }
//! ```
//!
//! ## See Also
//! - [`crate::traits::state_machine::state::State`]: The core trait implemented by [`SampleSecState`].
//! - [`crate::implementations::states::extract::validate_cik_format::ValidateCikFormat`]: A concrete, production-level state implementation that `SampleSecState` is modeled after.

use std::fmt;

use async_trait::async_trait;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::traits::state_machine::state::State;

pub mod context;
pub mod data;

pub use context::SampleSecStateContext;
pub use data::SampleSecStateInputData;
pub use data::SampleSecStateOutputData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleSecState {
    input: SampleSecStateInputData,
    context: SampleSecStateContext,
    output: Option<SampleSecStateOutputData>,
}

impl SampleSecState {
    #[must_use]
    pub const fn new(input: SampleSecStateInputData, context: SampleSecStateContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }
}

#[async_trait]
impl State for SampleSecState {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.output = Some(SampleSecStateOutputData {
            output_data: "Hello World!".to_string(),
        });
        Ok(())
    }
}

impl SMState for SampleSecState {
    type InputData = SampleSecStateInputData;
    type OutputData = SampleSecStateOutputData;
    type Context = SampleSecStateContext;

    fn get_state_name(&self) -> impl ToString {
        "Sample SEC State"
    }

    fn compute_output_data(&mut self) {}

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

impl fmt::Display for SampleSecState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.get_state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}
