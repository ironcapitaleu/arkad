//! # Sample SEC State Output Data
//!
//! This module defines the output data structure and updater patterns for the [`SampleSecState`](../../mod.rs) fixture.
//! It encapsulates a simple `String`-based output to demonstrate the basic requirements for state output data.
//!
//! ## Types
//! - [`SampleSecStateOutputData`]: Holds the output string produced by the sample state.
//! - [`SampleSecStateOutputDataUpdater`]: Updater type for modifying the output data.
//! - [`SampleSecStateOutputDataUpdaterBuilder`]: Builder for constructing updater instances.
//!
//! ## Integration
//! - Implements [`StateData`](crate::traits::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`SampleSecState`](../../mod.rs) to produce and store output data.
//!
//! ## See Also
//! - [`sec_input_data`](super::sec_input_data): The corresponding input data structure.
//! - [`crate::traits::state_machine::state::StateData`]: Trait for state data integration.
use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Output data for the `SampleSecState` fixture.
pub struct SampleSecStateOutputData {
    pub output_data: String,
}

impl SampleSecStateOutputData {
    /// Creates a new instance of the output data for the sample state.
    pub fn new(data: &(impl ToString + ?Sized)) -> Result<Self, StateError> {
        Ok(Self {
            output_data: data.to_string(),
        })
    }

    /// Returns a reference to the output data string.
    #[must_use]
    pub const fn output_data(&self) -> &String {
        &self.output_data
    }
}
impl StateData for SampleSecStateOutputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(input_data) = updates.output_data {
            self.output_data = input_data;
        }
        Ok(())
    }
}
impl SMStateData for SampleSecStateOutputData {
    type UpdateType = SampleSecStateOutputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl Default for SampleSecStateOutputData {
    fn default() -> Self {
        Self {
            output_data: String::from("Hello World!"),
        }
    }
}

impl fmt::Display for SampleSecStateOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tOutput Data: {}", self.output_data,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for [`SampleSecStateOutputData`].
pub struct SampleSecStateOutputDataUpdater {
    pub output_data: Option<String>,
}

/// Builder for [`SampleSecStateOutputDataUpdater`].
pub struct SampleSecStateOutputDataUpdaterBuilder {
    output_data: Option<String>,
}

impl SampleSecStateOutputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { output_data: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn output_data(mut self, output_data: &(impl ToString + ?Sized)) -> Self {
        self.output_data = Some(output_data.to_string());
        self
    }

    #[must_use]
    pub fn build(self) -> SampleSecStateOutputDataUpdater {
        SampleSecStateOutputDataUpdater {
            output_data: self.output_data,
        }
    }
}

impl Default for SampleSecStateOutputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
