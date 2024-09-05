use state_maschine::prelude::*;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct RetievalData {
    state_data: String,
}

impl StateData for RetievalData {
    type UpdateType = RetrievalDataUpdater;
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
pub struct RetrievalDataUpdater {
    pub state_data: Option<String>,
}
