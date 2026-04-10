use crate::state_machine::state::StateData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleStateData {
    state_data: String,
}

impl StateData for SampleStateData {
    type UpdateType = SampleStateDataUpdater;
    fn state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Some(value) = updates.state_data {
            self.state_data = value;
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleStateDataUpdater {
    pub state_data: Option<String>,
}
