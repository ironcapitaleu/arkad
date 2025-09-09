use std::fmt;

use state_maschine::prelude::ContextData as SMContextData;

use crate::traits::state_machine::state::ContextData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// State context for the `ExecuteSecRequest` state.
pub struct ExecuteSecRequestContext {
    pub data: String,
    pub max_retries: u32,
}

impl ExecuteSecRequestContext {
    /// Creates a new instance of the execute state context.
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

impl ContextData for ExecuteSecRequestContext {
    fn get_max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContextData for ExecuteSecRequestContext {
    type UpdateType = ExecuteSecRequestContextUpdater;

    fn get_context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(data) = updates.data {
            self.data = data;
        }
    }
}

impl Default for ExecuteSecRequestContext {
    fn default() -> Self {
        Self::new("Default Data")
    }
}

impl fmt::Display for ExecuteSecRequestContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context Data: {}", self.data)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the state context.
pub struct ExecuteSecRequestContextUpdater {
    pub data: Option<String>,
}

/// Builder for `ExecuteSecRequestContextUpdater`.
pub struct ExecuteSecRequestContextUpdaterBuilder {
    data: Option<String>,
}
impl ExecuteSecRequestContextUpdaterBuilder {
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
    pub fn build(self) -> ExecuteSecRequestContextUpdater {
        ExecuteSecRequestContextUpdater { data: self.data }
    }
}

impl Default for ExecuteSecRequestContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
