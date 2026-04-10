use crate::state_machine::state::Context;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SecondStateContext {
    context_data: String,
}

impl Context for SecondStateContext {
    type UpdateType = SecondStateContextUpdater;
    fn context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(value) = updates.context_data {
            self.context_data = value;
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SecondStateContextUpdater {
    pub context_data: Option<String>,
}
