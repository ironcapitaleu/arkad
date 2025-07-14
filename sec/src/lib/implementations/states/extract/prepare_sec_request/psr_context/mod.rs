use std::fmt;

use state_maschine::prelude::ContextData as SMContextData;

use crate::traits::state_machine::state::ContextData;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestContext {
    pub max_retries: u32,
}

impl PrepareSecRequestContext {
    #[must_use]
    pub const fn new() -> Self {
        Self { max_retries: 0 }
    }
}

impl ContextData for PrepareSecRequestContext {
    fn get_max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContextData for PrepareSecRequestContext {
    type UpdateType = PrepareSecRequestContextUpdater;

    fn get_context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl fmt::Display for PrepareSecRequestContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context Data:\nMax retries: {}", self.max_retries)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the state context.
pub struct PrepareSecRequestContextUpdater {
    pub max_retries: Option<u32>,
}

pub struct PrepareSecRequestContextUpdaterBuilder {
    max_retries: Option<u32>,
}
impl PrepareSecRequestContextUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { max_retries: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn data(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    #[must_use]
    pub const fn build(self) -> PrepareSecRequestContextUpdater {
        PrepareSecRequestContextUpdater {
            max_retries: self.max_retries,
        }
    }
}

impl Default for PrepareSecRequestContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
