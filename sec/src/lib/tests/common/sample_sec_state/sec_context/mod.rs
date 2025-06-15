use std::fmt;

use state_maschine::prelude::ContextData as SMContextData;

use crate::traits::state_machine::state::ContextData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleSecStateContext {
    pub data: String,
    pub max_retries: u32,
}

impl SampleSecStateContext {
    pub fn new(data: &(impl ToString + ?Sized)) -> Self {
        Self {
            data: data.to_string(),
            max_retries: 0,
        }
    }

    #[must_use]
    pub const fn data(&self) -> &String {
        &self.data
    }
}

impl ContextData for SampleSecStateContext {
    fn get_max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContextData for SampleSecStateContext {
    type UpdateType = SampleSecStateContextUpdater;

    fn get_context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(data) = updates.data {
            self.data = data;
        }
    }
}

impl Default for SampleSecStateContext {
    fn default() -> Self {
        Self::new("Default Data")
    }
}

impl fmt::Display for SampleSecStateContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context Data: {}", self.data)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleSecStateContextUpdater {
    pub data: Option<String>,
}

pub struct SampleSecStateContextUpdaterBuilder {
    data: Option<String>,
}
impl SampleSecStateContextUpdaterBuilder {
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
    pub fn build(self) -> SampleSecStateContextUpdater {
        SampleSecStateContextUpdater { data: self.data }
    }
}

impl Default for SampleSecStateContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::*;

    use super::{SampleSecStateContext, SampleSecStateContextUpdaterBuilder};

    #[test]
    fn should_return_reference_to_default_validation_context_when_initialized_with_default() {
        let validation_context = SampleSecStateContext::default();

        let expected_result = &SampleSecStateContext::default();

        let result = validation_context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_new_as_constructor() {
        let validation_context = &SampleSecStateContext::new("0000000000");

        let default_validation_context = &SampleSecStateContext::default();

        let result = validation_context.get_context();

        assert_ne!(result, default_validation_context);
    }

    #[test]
    fn should_update_context_data_to_specified_string_when_update_contains_specified_string() {
        let mut context = SampleSecStateContext::default();
        let update = SampleSecStateContextUpdaterBuilder::new()
            .data("Updated Data!")
            .build();

        let expected_result = &SampleSecStateContext::new("Updated Data!");

        context.update_context(update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut context = SampleSecStateContext::default();
        let update = SampleSecStateContextUpdaterBuilder::new()
            .data("First Data Update!")
            .data("Latest Data Update!")
            .build();

        let expected_result = &SampleSecStateContext::new("Latest Data Update!");

        context.update_context(update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_leave_context_data_the_default_when_update_contains_a_different_string() {
        let mut context = SampleSecStateContext::default();
        let update = SampleSecStateContextUpdaterBuilder::new()
            .data("Updated Data!")
            .build();

        context.update_context(update);
        let result = context.get_context().data();

        assert_ne!(result, "Default Data");
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = SampleSecStateContext::default();
        let empty_update = SampleSecStateContextUpdaterBuilder::default().build();

        let expected_result = &SampleSecStateContext::default();

        context.update_context(empty_update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }
}
