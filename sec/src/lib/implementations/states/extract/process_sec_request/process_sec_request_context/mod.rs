use std::fmt;

use state_maschine::prelude::ContextData as SMContextData;

use crate::traits::state_machine::state::ContextData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// State context for the `ProcessSecRequest` state.
pub struct ProcessSecRequestContext {
    pub data: String,
    pub max_retries: u32,
}

impl ProcessSecRequestContext {
    /// Creates a new instance of the process state context.
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

impl ContextData for ProcessSecRequestContext {
    fn get_max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContextData for ProcessSecRequestContext {
    type UpdateType = ProcessSecRequestContextUpdater;

    fn get_context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(data) = updates.data {
            self.data = data;
        }
    }
}

impl Default for ProcessSecRequestContext {
    fn default() -> Self {
        Self::new("Default Data")
    }
}

impl fmt::Display for ProcessSecRequestContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context Data: {}", self.data)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the state context.
pub struct ProcessSecRequestContextUpdater {
    pub data: Option<String>,
}

/// Builder for `ProcessSecRequestContextUpdater`.
pub struct ProcessSecRequestContextUpdaterBuilder {
    data: Option<String>,
}
impl ProcessSecRequestContextUpdaterBuilder {
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
    pub fn build(self) -> ProcessSecRequestContextUpdater {
        ProcessSecRequestContextUpdater { data: self.data }
    }
}

impl Default for ProcessSecRequestContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
