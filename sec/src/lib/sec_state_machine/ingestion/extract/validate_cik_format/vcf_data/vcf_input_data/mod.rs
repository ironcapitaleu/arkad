use state_maschine::prelude::*;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatInputData {
    cik: String,
}

impl StateData for ValidateCikFormatInputData {
    fn get_state(&self) -> &Self {
        &self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        self.cik = updates;
    }

    type UpdateType = String;
}
