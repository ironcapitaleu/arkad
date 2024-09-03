use crate::state_machine::state::ContextData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleStateContext {
    context_data: String,
}

impl SampleStateContext {
    pub fn new(context_data: String) -> Self {
        Self { context_data }
    }
}

impl ContextData for SampleStateContext {
    type UpdateType = SampleStateContextUpdater;
    fn get_context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(value) = updates.context_data {
            self.context_data = value;
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleStateContextUpdater {
    pub context_data: Option<String>,
}

pub struct SampleStateContextUpdaterBuilder {
    context_data: Option<String>,
}

impl SampleStateContextUpdaterBuilder {
    pub fn new() -> Self {
        Self { context_data: None }
    }

    pub fn context_data(mut self, context_data: String) -> Self {
        self.context_data = Some(context_data);
        self
    }

    pub fn build(self) -> SampleStateContextUpdater {
        SampleStateContextUpdater {
            context_data: self.context_data,
        }
    }
}

impl Default for SampleStateContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
