use crate::state_machine::sec_error::SecError;
use state_maschine::prelude::*;

pub trait SecStateData: StateData {
    /// Updates the state with new data given in the `updates` parameter.
    ///
    /// # Errors
    ///
    /// Returns a `SecError` if the update fails.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), SecError>;
}
