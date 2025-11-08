use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::sec_response::SecResponse;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Input data for the `ValidateSecResponse` fixture.
pub struct ValidateSecResponseInputData {
    pub sec_response: SecResponse,
}

impl ValidateSecResponseInputData {
    /// Creates a new instance of the input data for the validate SEC response state.
    pub fn new(sec_response: SecResponse) -> Self {
        Self { sec_response }
    }

    /// Returns a reference to the raw input string.
    #[must_use]
    pub const fn sec_response(&self) -> &SecResponse {
        &self.sec_response
    }
}

impl StateData for ValidateSecResponseInputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(sec_response) = updates.sec_response {
            self.sec_response = sec_response;
        }
        Ok(())
    }
}

impl SMStateData for ValidateSecResponseInputData {
    type UpdateType = ValidateSecResponseInputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl Default for ValidateSecResponseInputData {
    fn default() -> Self {
        Self {
            sec_response: SecResponse::default(),
        }
    }
}

impl fmt::Display for ValidateSecResponseInputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tInput Data: {}", self.sec_response())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for [`ValidateSecResponseInputData`].
pub struct ValidateSecResponseInputDataUpdater {
    pub sec_response: Option<SecResponse>,
}

/// Builder for [`ValidateSecResponseInputDataUpdater`].
pub struct ValidateSecResponseInputDataUpdaterBuilder {
    sec_response: Option<SecResponse>,
}
impl ValidateSecResponseInputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { sec_response: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn sec_response(mut self, sec_response: &SecResponse) -> Self {
        self.sec_response = Some(sec_response.clone());
        self
    }

    #[must_use]
    pub fn build(self) -> ValidateSecResponseInputDataUpdater {
        ValidateSecResponseInputDataUpdater {
            sec_response: self.sec_response,
        }
    }
}

impl Default for ValidateSecResponseInputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
