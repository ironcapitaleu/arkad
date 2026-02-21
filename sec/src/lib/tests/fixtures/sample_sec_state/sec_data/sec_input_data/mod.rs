//! # Sample SEC State Input
//!
//! This module defines the input structure and updater patterns for the [`SampleSecState`](../../mod.rs) fixture.
//! It provides a simple `String`-based input to demonstrate the basic requirements for state input.
//!
//! ## Types
//! - [`SampleSecStateInput`]: Holds the unvalidated input string to be processed by the sample state.
//! - [`SampleSecStateInputUpdater`]: Updater type for modifying the input.
//! - [`SampleSecStateInputUpdaterBuilder`]: Builder for constructing updater instances.
//!
//! ## Integration
//! - Implements [`StateData`](crate::traits::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`SampleSecState`](../../mod.rs) to receive input data.
//!
//! ## See Also
//! - [`sec_output_data`](super::sec_output_data): The corresponding output data structure.
//! - [`crate::traits::state_machine::state::StateData`]: Trait for state data integration.
use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Input data for the `SampleSecState` fixture.
pub struct SampleSecStateInput {
    pub input_data: String,
}

impl SampleSecStateInput {
    /// Creates a new instance of the input data for the sample state.
    pub fn new(input_data: &(impl ToString + ?Sized)) -> Self {
        Self {
            input_data: input_data.to_string(),
        }
    }

    /// Returns a reference to the raw input string.
    #[must_use]
    pub const fn input_data(&self) -> &String {
        &self.input_data
    }
}

impl StateData for SampleSecStateInput {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(input_data) = updates.input_data {
            self.input_data = input_data;
        }
        Ok(())
    }
}

impl SMStateData for SampleSecStateInput {
    type UpdateType = SampleSecStateInputUpdater;

    fn state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl Default for SampleSecStateInput {
    fn default() -> Self {
        Self {
            input_data: "Hello".to_string(),
        }
    }
}

impl fmt::Display for SampleSecStateInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tInput Data: {}", self.input_data,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for [`SampleSecStateInput`].
pub struct SampleSecStateInputUpdater {
    pub input_data: Option<String>,
}

/// Builder for [`SampleSecStateInputUpdater`].
pub struct SampleSecStateInputUpdaterBuilder {
    input_data: Option<String>,
}
impl SampleSecStateInputUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { input_data: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn input_data(mut self, input_data: &(impl ToString + ?Sized)) -> Self {
        self.input_data = Some(input_data.to_string());
        self
    }

    #[must_use]
    pub fn build(self) -> SampleSecStateInputUpdater {
        SampleSecStateInputUpdater {
            input_data: self.input_data,
        }
    }
}

impl Default for SampleSecStateInputUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
