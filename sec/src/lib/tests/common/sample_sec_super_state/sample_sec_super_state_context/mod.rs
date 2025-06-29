use crate::traits::state_machine::state::ContextData;
use state_maschine::prelude::ContextData as SMContextData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleSecSuperStateContext;

impl ContextData for SampleSecSuperStateContext {
    fn get_max_retries(&self) -> u32 {
        0
    }
}

impl SMContextData for SampleSecSuperStateContext {
    type UpdateType = ();

    fn get_context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, _updates: Self::UpdateType) {}
}
