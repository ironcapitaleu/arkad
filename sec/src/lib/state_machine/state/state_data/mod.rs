use crate::error::State as StateError;
use state_maschine::prelude::StateData as SMStateData;

pub trait StateData: SMStateData {
    /// Updates the state with new data given in the `updates` parameter.
    ///
    /// # Errors
    ///
    /// Returns a `crate::error::State` if the update fails.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError>;
}
