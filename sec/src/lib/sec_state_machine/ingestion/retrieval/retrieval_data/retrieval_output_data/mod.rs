use state_maschine::prelude::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct RetrievalOutputData {
    response: String,
}

impl RetrievalOutputData {
    pub fn new(response: &(impl ToString + ?Sized)) -> Self {
        Self {
            response: response.to_string(),
        }
    }

    #[must_use]
    pub const fn response(&self) -> &String {
        &self.response
    }
}

impl Default for RetrievalOutputData {
    fn default() -> Self {
        Self::new("")
    }
}

impl fmt::Display for RetrievalOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tResponse: {}", self.response(),)
    }
}

impl StateData for RetrievalOutputData {
    type UpdateType = RetrievalOutputDataUpdater;
    fn get_state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Some(value) = updates.response {
            self.response = value;
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct RetrievalOutputDataUpdater {
    pub response: Option<String>,
}

pub struct RetrievalOutputDataUpdaterBuilder {
    response: Option<String>,
}
impl RetrievalOutputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { response: None }
    }

    #[must_use]
    pub fn response(mut self, response: &(impl ToString + ?Sized)) -> Self {
        self.response = Some(response.to_string());
        self
    }

    #[must_use]
    pub fn build(self) -> RetrievalOutputDataUpdater {
        RetrievalOutputDataUpdater {
            response: self.response,
        }
    }
}

impl Default for RetrievalOutputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{RetrievalOutputData, RetrievalOutputDataUpdaterBuilder};
    use state_maschine::prelude::*;

    #[test]
    fn should_return_reference_to_default_retrieval_state_data_when_initialized_with_default() {
        let retrieval_state_data = &RetrievalOutputData::default();

        let expected_result = &RetrievalOutputData::default();

        let result = retrieval_state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let retrieval_state_data = &RetrievalOutputData::new("Demir ist der Boss.");

        let default_retrieval_state_data = &RetrievalOutputData::default();

        let result = retrieval_state_data.get_state();

        assert_ne!(result, default_retrieval_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_string_when_update_contains_specified_string() {
        let mut state_data = RetrievalOutputData::default();
        let update = RetrievalOutputDataUpdaterBuilder::default()
            .response("Updated State!")
            .build();

        let expected_result = &RetrievalOutputData::new("Updated State!");

        state_data.update_state(update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut state_data = RetrievalOutputData::default();
        let update = RetrievalOutputDataUpdaterBuilder::default()
            .response("First Update!")
            .response("Latest Update!")
            .build();

        let expected_result = &RetrievalOutputData::new("Latest Update!");

        state_data.update_state(update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = RetrievalOutputData::default();
        let empty_update = RetrievalOutputDataUpdaterBuilder::default().build();

        let expected_result = &RetrievalOutputData::default();

        state_data.update_state(empty_update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_empty_string_when_retrieval_output_data_initialized_with_default() {
        let retrieval_state_data = &RetrievalOutputData::default();

        let expected_result = "";

        let result = retrieval_state_data.get_state().response();

        assert_eq!(result, expected_result);
    }
}
