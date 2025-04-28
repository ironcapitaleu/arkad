use crate::state_machine::state::StateData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleStateData {
    state_data: String,
}

impl SampleStateData {
    pub const fn new(state_data: String) -> Self {
        Self { state_data }
    }
}

impl StateData for SampleStateData {
    type UpdateType = SampleStateDataUpdater;
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
pub struct SampleStateDataUpdater {
    pub state_data: Option<String>,
}

pub struct SampleStateDataUpdaterBuilder {
    state_data: Option<String>,
}

impl SampleStateDataUpdaterBuilder {
    pub const fn new() -> Self {
        Self { state_data: None }
    }

    pub fn state_data(mut self, state_data: String) -> Self {
        self.state_data = Some(state_data);
        self
    }

    pub fn build(self) -> SampleStateDataUpdater {
        SampleStateDataUpdater {
            state_data: self.state_data,
        }
    }
}

impl Default for SampleStateDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
