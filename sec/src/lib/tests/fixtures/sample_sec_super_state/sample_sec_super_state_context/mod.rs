use crate::traits::state_machine::state::Context;
use state_maschine::prelude::Context as SMContext;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleSecSuperStateContext;

impl Context for SampleSecSuperStateContext {
    fn max_retries(&self) -> u32 {
        0
    }
}

impl SMContext for SampleSecSuperStateContext {
    type UpdateType = ();

    fn context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, _updates: Self::UpdateType) {}
}
