use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestInputData {
    pub input_data: String,
}

impl PrepareSecRequestInputData {
    pub fn new(input_data: &(impl ToString + ?Sized)) -> Self {
        Self {
            input_data: input_data.to_string(),
        }
    }

    #[must_use]
    pub const fn input_data(&self) -> &String {
        &self.input_data
    }
}

impl StateData for PrepareSecRequestInputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(input_data) = updates.input_data {
            self.input_data = input_data;
        }
        Ok(())
    }
}

impl SMStateData for PrepareSecRequestInputData {
    type UpdateType = PrepareSecRequestInputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl Default for PrepareSecRequestInputData {
    fn default() -> Self {
        Self {
            input_data: "Hello".to_string(),
        }
    }
}

impl fmt::Display for PrepareSecRequestInputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tInput Data: {}", self.input_data,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestInputDataUpdater {
    pub input_data: Option<String>,
}

pub struct PrepareSecRequestInputDataUpdaterBuilder {
    input_data: Option<String>,
}
impl PrepareSecRequestInputDataUpdaterBuilder {
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
    pub fn build(self) -> PrepareSecRequestInputDataUpdater {
        PrepareSecRequestInputDataUpdater {
            input_data: self.input_data,
        }
    }
}

impl Default for PrepareSecRequestInputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
