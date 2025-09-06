use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Output data for the `ProcessSecRequest` fixture.
pub struct ProcessSecRequestOutputData {
    pub output_data: String,
}

impl ProcessSecRequestOutputData {
    /// Creates a new instance of the output data for the process SEC request.
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
impl StateData for ProcessSecRequestOutputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(input_data) = updates.output_data {
            self.output_data = input_data;
        }
        Ok(())
    }
}
impl SMStateData for ProcessSecRequestOutputData {
    type UpdateType = ProcessSecRequestOutputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl Default for ProcessSecRequestOutputData {
    fn default() -> Self {
        Self {
            output_data: String::from("Hello World!"),
        }
    }
}

impl fmt::Display for ProcessSecRequestOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tOutput Data: {}", self.output_data,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for [`ProcessSecRequestOutputData`].
pub struct ProcessSecRequestOutputDataUpdater {
    pub output_data: Option<String>,
}

/// Builder for [`ProcessSecRequestOutputDataUpdater`].
pub struct ProcessSecRequestOutputDataUpdaterBuilder {
    output_data: Option<String>,
}

impl ProcessSecRequestOutputDataUpdaterBuilder {
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
    pub fn build(self) -> ProcessSecRequestOutputDataUpdater {
        ProcessSecRequestOutputDataUpdater {
            output_data: self.output_data,
        }
    }
}

impl Default for ProcessSecRequestOutputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
