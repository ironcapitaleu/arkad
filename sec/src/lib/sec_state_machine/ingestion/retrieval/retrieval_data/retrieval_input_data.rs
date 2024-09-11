use crate::sec_state_machine::ingestion::retrieval::retrieval_context::config::DEFAULT_CIK;
use state_maschine::prelude::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct RetrievalInputData {
    cik: String,
}

impl RetrievalInputData {
    pub fn new(cik: &(impl ToString + ?Sized)) -> Self {
        Self {
            cik: cik.to_string(),
        }
    }

    #[must_use]
    pub const fn cik(&self) -> &String {
        &self.cik
    }
}

impl Default for RetrievalInputData {
    fn default() -> Self {
        Self::new(DEFAULT_CIK)
    }
}

impl fmt::Display for RetrievalInputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tCIK: {}", self.cik(),)
    }
}

impl StateData for RetrievalInputData {
    type UpdateType = RetrievalInputDataUpdater;
    fn get_state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Some(value) = updates.cik {
            self.cik = value;
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct RetrievalInputDataUpdater {
    pub cik: Option<String>,
}

pub struct RetrievalInputDataUpdaterBuilder {
    cik: Option<String>,
}
impl RetrievalInputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { cik: None }
    }

    #[must_use]
    pub fn state_data(mut self, cik: &(impl ToString + ?Sized)) -> Self {
        self.cik = Some(cik.to_string());
        self
    }

    #[must_use]
    pub fn build(self) -> RetrievalInputDataUpdater {
        RetrievalInputDataUpdater { cik: self.cik }
    }
}

impl Default for RetrievalInputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{RetrievalInputData, RetrievalInputDataUpdaterBuilder, DEFAULT_CIK};
    use state_maschine::prelude::*;

    #[test]
    fn should_return_reference_to_default_retrieval_state_data_when_initialized_with_default() {
        let retrieval_state_data = &RetrievalInputData::default();

        let expected_result = &RetrievalInputData::default();

        let result = retrieval_state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let retrieval_state_data = &RetrievalInputData::new("Demir ist der Boss.");

        let default_retrieval_state_data = &RetrievalInputData::default();

        let result = retrieval_state_data.get_state();

        assert_ne!(result, default_retrieval_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_string_when_update_contains_specified_string() {
        let mut state_data = RetrievalInputData::default();
        let update = RetrievalInputDataUpdaterBuilder::default()
            .state_data("Updated State!")
            .build();

        let expected_result = &RetrievalInputData::new("Updated State!");

        state_data.update_state(update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut state_data = RetrievalInputData::default();
        let update = RetrievalInputDataUpdaterBuilder::default()
            .state_data("First Update!")
            .state_data("Latest Update!")
            .build();

        let expected_result = &RetrievalInputData::new("Latest Update!");

        state_data.update_state(update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = RetrievalInputData::default();
        let empty_update = RetrievalInputDataUpdaterBuilder::default().build();

        let expected_result = &RetrievalInputData::default();

        state_data.update_state(empty_update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_initialized_as_string_when_retrieval_data_initialized_with_default() {
        let retrieval_state_data = &RetrievalInputData::default();

        let expected_result = DEFAULT_CIK;

        let result = retrieval_state_data.get_state().cik();

        assert_eq!(result, expected_result);
    }
}
