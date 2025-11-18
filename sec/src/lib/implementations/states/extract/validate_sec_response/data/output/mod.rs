use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::shared::validated_sec_response::ValidatedSecResponse;
use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Output data for `ValidateSecResponse`.
#[derive(Default)]
pub struct ValidateSecResponseOutputData {
    pub validated_sec_response: ValidatedSecResponse,
}

impl ValidateSecResponseOutputData {
    /// Creates a new instance of the output data for the validate SEC response state.
    /// 
    /// # Errors
    /// Returns a `StateError` if the provided data is invalid.
    pub const fn new(validated_sec_response: ValidatedSecResponse) -> Result<Self, StateError> {
        Ok(Self {
            validated_sec_response,
        })
    }

    /// Returns a reference to the output data string.
    #[must_use]
    pub const fn validated_sec_response(&self) -> &ValidatedSecResponse {
        &self.validated_sec_response
    }
}
impl StateData for ValidateSecResponseOutputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(validated_sec_response) = updates.validated_sec_response {
            self.validated_sec_response = validated_sec_response;
        }
        Ok(())
    }
}
impl SMStateData for ValidateSecResponseOutputData {
    type UpdateType = ValidateSecResponseOutputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, _updates: Self::UpdateType) {}
}


impl fmt::Display for ValidateSecResponseOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tOutput Data: {}", self.validated_sec_response,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for [`ValidateSecResponseOutputData`].
pub struct ValidateSecResponseOutputDataUpdater {
    pub validated_sec_response: Option<ValidatedSecResponse>,
}

/// Builder for [`ValidateSecResponseOutputDataUpdater`].
pub struct ValidateSecResponseOutputDataUpdaterBuilder {
    validated_sec_response: Option<ValidatedSecResponse>,
}

impl ValidateSecResponseOutputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { validated_sec_response: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn validated_sec_response(mut self, validated_sec_response: ValidatedSecResponse) -> Self {
        self.validated_sec_response = Some(validated_sec_response);
        self
    }

    #[must_use]
    pub fn build(self) -> ValidateSecResponseOutputDataUpdater {
        ValidateSecResponseOutputDataUpdater {
            validated_sec_response: self.validated_sec_response,
        }
    }
}

impl Default for ValidateSecResponseOutputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
