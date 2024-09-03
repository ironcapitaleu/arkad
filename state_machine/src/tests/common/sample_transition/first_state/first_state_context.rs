use crate::state_machine::state::ContextData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct FirstStateContext {
    context_data: String,
}

impl ContextData for FirstStateContext {
    type UpdateType = FirstStateContextUpdater;
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
pub struct FirstStateContextUpdater {
    pub context_data: Option<String>,
}
