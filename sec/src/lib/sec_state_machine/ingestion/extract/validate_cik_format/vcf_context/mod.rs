use state_maschine::prelude::*;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatContext {
    status: Status,
    cik: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
enum Status {
    PreValidation,
    PostValidation,
}

impl ContextData for ValidateCikFormatContext {
    type UpdateType = String;
    fn get_context(&self) -> &Self {
        &self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        self.cik = updates;
    }
}
