use crate::sec_state_machine::sec_error::SecError;
use state_maschine::prelude::*;

pub trait SecStateData: StateData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), SecError>;
}
