use std::fmt;

use state_maschine::prelude::ContextData as SMContextData;

use crate::traits::state_machine::state::ContextData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequestContext {
    pub data: String,
    pub max_retries: u32,
}

impl PrepareSecRequestContext {
    pub fn new(data: &(impl ToString + ?Sized)) -> Self {
        Self {
            data: data.to_string(),
            max_retries: 0,
        }
    }

    /// Returns a reference to the context's inner data string.
    #[must_use]
    pub const fn data(&self) -> &String {
        &self.data
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
        if let Some(data) = updates.data {
            self.data = data;
        }
    }
}

impl Default for PrepareSecRequestContext {
    fn default() -> Self {
        Self::new("Default Data")
    }
}

impl fmt::Display for PrepareSecRequestContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context Data: {}", self.data)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the state context.
pub struct PrepareSecRequestContextUpdater {
    pub data: Option<String>,
}

pub struct PrepareSecRequestContextUpdaterBuilder {
    data: Option<String>,
}
impl PrepareSecRequestContextUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { data: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn data(mut self, data: &(impl ToString + ?Sized)) -> Self {
        self.data = Some(data.to_string());
        self
    }

    #[must_use]
    pub fn build(self) -> PrepareSecRequestContextUpdater {
        PrepareSecRequestContextUpdater { data: self.data }
    }
}

impl Default for PrepareSecRequestContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
