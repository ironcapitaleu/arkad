use state_maschine::prelude::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct RetrievalData {
    status: Status,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum Status {
    PreRetrieval,
    PostRetrieval,
}

impl Status {
    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::PreRetrieval | Self::PostRetrieval => Self::PostRetrieval,
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status_str = match self {
            Self::PreRetrieval => "Pre-Retrieval",
            Self::PostRetrieval => "Post-Retrieval",
        };
        write!(f, "{status_str}")
    }
}

impl RetrievalData {
    #[must_use]
    pub const fn new(status: Status) -> Self {
        Self { status }
    }

    #[must_use]
    pub const fn status(&self) -> &Status {
        &self.status
    }
}

impl Default for RetrievalData {
    fn default() -> Self {
        Self::new(Status::PreRetrieval)
    }
}

impl fmt::Display for RetrievalData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tStatus: {}", self.status())
    }
}

impl StateData for RetrievalData {
    type UpdateType = RetrievalDataUpdater;
    fn get_state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Some(value) = updates.status {
            self.status = value;
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct RetrievalDataUpdater {
    pub status: Option<Status>,
}

pub struct RetrievalDataUpdaterBuilder {
    status: Option<Status>,
}
impl RetrievalDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { status: None }
    }

    #[must_use]
    pub const fn state_data(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    #[must_use]
    pub const fn build(self) -> RetrievalDataUpdater {
        RetrievalDataUpdater {
            status: self.status,
        }
    }
}

impl Default for RetrievalDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{RetrievalData, RetrievalDataUpdaterBuilder, Status};
    use state_maschine::prelude::*;

    #[test]
    fn should_return_reference_to_default_retrieval_state_data_when_initialized_with_default() {
        let retrieval_state_data = &RetrievalData::default();

        let expected_result = &RetrievalData::default();

        let result = retrieval_state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let retrieval_state_data = &RetrievalData::new(Status::PostRetrieval);

        let default_retrieval_state_data = &RetrievalData::default();

        let result = retrieval_state_data.get_state();

        assert_ne!(result, default_retrieval_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_status_when_update_contains_specified_status() {
        let mut state_data = RetrievalData::default();
        let update = RetrievalDataUpdaterBuilder::default()
            .state_data(Status::PostRetrieval)
            .build();

        let expected_result = &RetrievalData::new(Status::PostRetrieval);

        state_data.update_state(update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_status_when_multiple_updates_in_builder() {
        let mut state_data = RetrievalData::default();
        let update = RetrievalDataUpdaterBuilder::default()
            .state_data(Status::PreRetrieval)
            .state_data(Status::PostRetrieval)
            .build();

        let expected_result = &RetrievalData::new(Status::PostRetrieval);

        state_data.update_state(update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = RetrievalData::default();
        let empty_update = RetrievalDataUpdaterBuilder::default().build();

        let expected_result = &RetrievalData::default();

        state_data.update_state(empty_update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_initialized_as_string_when_retrieval_data_initialized_with_default() {
        let retrieval_state_data = &RetrievalData::default();

        let expected_result = "Pre-Retrieval";

        let result = retrieval_state_data.get_state().status().to_string();

        assert_eq!(result, expected_result);
    }
}
