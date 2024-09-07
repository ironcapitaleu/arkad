use state_maschine::prelude::*;
pub mod config;

pub use config::get_sec_user_client;
use config::{get_sec_user_agent, DEFAULT_CIK};

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
    pub fn cik(&self) -> &String {
        &self.cik
    }

    pub fn user_agent(&self) -> &String {
        &self.user_agent
    }
}

impl Default for RetrievalContext {
    fn default() -> Self {
        Self::new(get_sec_user_agent().to_string(), DEFAULT_CIK)
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

#[cfg(test)]
mod tests {
    use crate::sec_state_machine::ingestion::retrieval::retrieval_context::config::{get_sec_user_agent, DEFAULT_CIK};

    use super::{RetrievalContext, RetrievalContextUpdaterBuilder};
    use state_maschine::prelude::*;

    #[test]
    fn should_return_reference_to_default_retrieval_context_when_initialized_with_default() {
        let retrieval_context = &RetrievalContext::default();

        let expected_result = &RetrievalContext::default();

        let result = retrieval_context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_new_as_constructor() {
        let retrieval_context =
            &RetrievalContext::new("custom_user_agent@example.com", "Demir ist der Boss.");

        let default_retrieval_context = &RetrievalContext::default();

        let result = retrieval_context.get_context();

        assert_ne!(result, default_retrieval_context);
    }

    #[test]
    fn should_update_context_cik_data_to_specified_string_when_update_contains_specified_string() {
        let default_user_agent = get_sec_user_agent();
        let mut context = RetrievalContext::default();
        let update = RetrievalContextUpdaterBuilder::new()
            .cik(String::from("Updated CIK!"))
            .build();

        let expected_result = &RetrievalContext::new(default_user_agent, "Updated CIK!");

        context.update_context(update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_to_latest_specified_string_when_multiple_updates_in_builder() {
        let default_user_agent = get_sec_user_agent();
        let mut context = RetrievalContext::default();
        let update = RetrievalContextUpdaterBuilder::new()
            .cik(String::from("First CIK Update!"))
            .cik(String::from("Latest CIK Update!"))
            .build();

        let expected_result = &RetrievalContext::new(default_user_agent, "Latest CIK Update!");

        context.update_context(update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_leave_context_cik_data_the_default_when_update_contains_a_different_string() {
        let mut context = RetrievalContext::default();
        let update = RetrievalContextUpdaterBuilder::new()
            .cik(String::from("Updated CIK!"))
            .build();

        context.update_context(update);
        let result = context.get_context().cik();

        assert_ne!(result, DEFAULT_CIK);
    }

    #[test]
    fn should_update_user_agent_when_update_contains_new_user_agent() {
        let mut context = RetrievalContext::default();
        let user_agent_update = RetrievalContextUpdaterBuilder::new()
            .user_agent("updated_user_agent@example.com")
            .build();

        let expected_result = "updated_user_agent@example.com";

        context.update_context(user_agent_update);
        let result = context.get_context().user_agent();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_change_user_agent_when_update_contains_new_user_agent() {
        let mut context = RetrievalContext::default();
        let user_agent_update = RetrievalContextUpdaterBuilder::new()
            .user_agent("updated_user_agent@example.com")
            .build();

        let default_retrieval_context = &RetrievalContext::default();

        context.update_context(user_agent_update);
        let result = context.get_context();

        assert_ne!(result, default_retrieval_context);
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = RetrievalContext::default();
        let empty_update = RetrievalContextUpdaterBuilder::default().build();

        let expected_result = &RetrievalContext::default();

        context.update_context(empty_update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }
}
