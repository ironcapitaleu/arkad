use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::sec_response::SecResponse;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Output data for the `ExecuteSecRequest` fixture.
#[derive(Default)]
pub struct ExecuteSecRequestOutputData {
    pub response: SecResponse,
}

impl ExecuteSecRequestOutputData {
    /// Creates a new instance of the output data for the execute SEC request.
    /// 
    /// # Errors
    /// Returns `StateError` if the provided `SecResponse` is invalid.
    pub const fn new(response: SecResponse) -> Result<Self, StateError> {
        Ok(Self { response })
    }

    /// Returns a reference to the response.
    #[must_use]
    pub const fn response(&self) -> &SecResponse {
        &self.response
    }
}
impl StateData for ExecuteSecRequestOutputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(response) = updates.response {
            self.response = response;
        }
        Ok(())
    }
}
impl SMStateData for ExecuteSecRequestOutputData {
    type UpdateType = ExecuteSecRequestOutputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, _updates: Self::UpdateType) {}
}


impl fmt::Display for ExecuteSecRequestOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\t{}", self.response)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for [`ExecuteSecRequestOutputData`].
pub struct ExecuteSecRequestOutputDataUpdater {
    pub response: Option<SecResponse>,
}

/// Builder for [`ExecuteSecRequestOutputDataUpdater`].
pub struct ExecuteSecRequestOutputDataUpdaterBuilder {
    response: Option<SecResponse>,
}

impl ExecuteSecRequestOutputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { response: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn response(mut self, response: SecResponse) -> Self {
        self.response = Some(response);
        self
    }

    #[must_use]
    pub fn build(self) -> ExecuteSecRequestOutputDataUpdater {
        ExecuteSecRequestOutputDataUpdater {
            response: self.response,
        }
    }
}

impl Default for ExecuteSecRequestOutputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
