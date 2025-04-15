use state_maschine::prelude::*;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatOutputData {
    validation_result: String,
}

impl StateData for ValidateCikFormatOutputData {
    fn get_state(&self) -> &Self {
        &self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        self.validation_result = updates;
    }

    type UpdateType = String;
}
