use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleSecStateOutputData {
    pub output_data: String,
}

impl SampleSecStateOutputData {
    pub fn new(data: &(impl ToString + ?Sized)) -> Result<Self, StateError> {
        Ok(Self {
            output_data: data.to_string(),
        })
    }

    #[must_use]
    pub const fn output_data(&self) -> &String {
        &self.output_data
    }
}
impl StateData for SampleSecStateOutputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(input_data) = updates.output_data {
            self.output_data = input_data;
        }
        Ok(())
    }
}
impl SMStateData for SampleSecStateOutputData {
    type UpdateType = SampleSecStateOutputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }
    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

impl Default for SampleSecStateOutputData {
    fn default() -> Self {
        Self {
            output_data: String::from("Hello World!"),
        }
    }
}

impl fmt::Display for SampleSecStateOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tOutput Data: {}", self.output_data,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleSecStateOutputDataUpdater {
    pub output_data: Option<String>,
}

pub struct SampleSecStateOutputDataUpdaterBuilder {
    output_data: Option<String>,
}

impl SampleSecStateOutputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { output_data: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn output_data(mut self, output_data: &(impl ToString + ?Sized)) -> Self {
        self.output_data = Some(String::from(output_data.to_string()));
        self
    }

    #[must_use]
    pub fn build(self) -> SampleSecStateOutputDataUpdater {
        SampleSecStateOutputDataUpdater {
            output_data: self.output_data,
        }
    }
}

impl Default for SampleSecStateOutputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::StateData as SMStateData;

    use super::{SampleSecStateOutputData, SampleSecStateOutputDataUpdaterBuilder};
    use crate::traits::state_machine::state::StateData;

    #[test]
    fn should_return_reference_to_default_validation_state_data_when_initialized_with_default() {
        let validation_state_data = SampleSecStateOutputData::default();

        let expected_result = &SampleSecStateOutputData::default();

        let result = validation_state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let validation_state_data = &SampleSecStateOutputData::new("12345")
            .expect("Provided hardcoded output data should always be valid");

        let default_validation_state_data = &SampleSecStateOutputData::default();

        let result = validation_state_data.get_state();

        assert_ne!(result, default_validation_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_string_when_update_contains_specified_string() {
        let mut state_data = SampleSecStateOutputData::default();
        let update = SampleSecStateOutputDataUpdaterBuilder::default()
            .output_data("12345")
            .build();

        let expected_result = &SampleSecStateOutputData::new("12345")
            .expect("Provided hardcoded output data should always be valid");

        StateData::update_state(&mut state_data, update)
            .expect("Provided hardcoded update should succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut state_data = SampleSecStateOutputData::default();
        let update = SampleSecStateOutputDataUpdaterBuilder::default()
            .output_data("12345")
            .output_data("67890")
            .build();

        let expected_result = &SampleSecStateOutputData::new("67890")
            .expect("Provided hardcoded output data should always be valid.");

        StateData::update_state(&mut state_data, update)
            .expect("Provided hardcoded update should succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = SampleSecStateOutputData::default();
        let empty_update = SampleSecStateOutputDataUpdaterBuilder::default().build();

        let expected_result = &SampleSecStateOutputData::default();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Provided hardcoded update should succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_formatted_and_validated_default_string_when_validation_output_data_initialized_with_default()
     {
        let validation_state_data = &SampleSecStateOutputData::default();
        let output_string = String::from("Hello World!");

        let expected_result = &output_string;

        let result = validation_state_data.get_state().output_data();

        assert_eq!(result, expected_result);
    }
}
