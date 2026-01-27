use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;
use state_maschine::prelude::StateData as SMStateData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleSecSuperStateData;

impl StateData for SampleSecSuperStateData {
    fn update_state(&mut self, _updates: Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

impl SMStateData for SampleSecSuperStateData {
    type UpdateType = ();

    fn state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, _updates: Self::UpdateType) {}
}
