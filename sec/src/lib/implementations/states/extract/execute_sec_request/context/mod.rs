use std::fmt;

use state_maschine::prelude::ContextData as SMContextData;
use crate::shared::cik::Cik;
use crate::traits::state_machine::state::ContextData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// State context for the `ExecuteSecRequest` state.
pub struct ExecuteSecRequestContext {
    pub cik: Cik,
    pub max_retries: u32,
}

impl ExecuteSecRequestContext {
    /// Creates a new instance of the execute state context.
    pub fn new(cik: Cik) -> Self {
        Self {
            cik,
            max_retries: 0,
        }
    }

    /// Returns a reference to the context's inner data string.
    #[must_use]
    pub const fn cik(&self) -> &Cik {
        &self.cik
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
        if let Some(cik) = updates.cik {
            self.cik = cik;
        }
    }
}

impl Default for ExecuteSecRequestContext {
    fn default() -> Self {
        Self::new(Cik::default())
    }
}

impl fmt::Display for ExecuteSecRequestContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context Data: {}", self.cik)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the state context.
pub struct ExecuteSecRequestContextUpdater {
    pub cik: Option<Cik>,
}

/// Builder for `ExecuteSecRequestContextUpdater`.
pub struct ExecuteSecRequestContextUpdaterBuilder {
    cik: Option<Cik>,
}
impl ExecuteSecRequestContextUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { cik: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: Cik) -> Self {
        self.cik = Some(cik);
        self
    }

    #[must_use]
    pub fn build(self) -> ExecuteSecRequestContextUpdater {
        ExecuteSecRequestContextUpdater { cik: self.cik }
    }
}

impl Default for ExecuteSecRequestContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
