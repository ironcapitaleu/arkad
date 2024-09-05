use state_maschine::prelude::*;
pub mod config;

pub use config::{get_sec_user_agent, DEFAULT_CIK};

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct RetrievalContext {
    user_agent: String,
    cik: String,
}

impl RetrievalContext {
    pub fn new(user_agent: impl ToString, cik: impl ToString) -> Self {
        Self {
            user_agent: user_agent.to_string(),
            cik: cik.to_string(),
        }
    }
}

impl Default for RetrievalContext {
    fn default() -> Self {
        Self::new(get_sec_user_agent(), DEFAULT_CIK)
    }
}

impl ContextData for RetrievalContext {
    type UpdateType = RetrievalContextUpdater;
    fn get_context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(value) = updates.user_agent {
            self.user_agent = value;
        }
        if let Some(value) = updates.cik {
            self.cik = value;
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct RetrievalContextUpdater {
    pub user_agent: Option<String>,
    pub cik: Option<String>,
}

pub struct RetrievalContextUpdaterBuilder {
    user_agent: Option<String>,
    cik: Option<String>,
}

impl RetrievalContextUpdaterBuilder {
    pub fn new() -> Self {
        Self {
            user_agent: None,
            cik: None,
        }
    }

    pub fn user_agent(mut self, user_agent: impl ToString) -> Self {
        self.user_agent = Some(user_agent.to_string());
        self
    }

    pub fn cik(mut self, cik: impl ToString) -> Self {
        self.cik = Some(cik.to_string());
        self
    }

    pub fn build(self) -> RetrievalContextUpdater {
        RetrievalContextUpdater {
            user_agent: self.user_agent,
            cik: self.cik,
        }
    }
}

impl Default for RetrievalContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
