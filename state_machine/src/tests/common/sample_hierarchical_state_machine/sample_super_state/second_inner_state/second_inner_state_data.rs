use crate::state_machine::state::StateData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SecondInnerStateData {
    state_data: String,
}

impl StateData for SecondInnerStateData {
    type UpdateType = SecondInnerStateDataUpdater;
    fn get_state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Some(value) = updates.state_data {
            self.state_data = value;
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SecondInnerStateDataUpdater {
    pub state_data: Option<String>,
}
