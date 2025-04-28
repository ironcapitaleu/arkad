use std::fmt;

use state_maschine::prelude::*;

pub mod cik;
pub use cik::Cik;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatOutputData {
    pub validated_cik: Cik,
}

impl ValidateCikFormatOutputData {
    pub fn new(cik: &(impl ToString + ?Sized)) -> Self {
        Self {
            validated_cik: Cik::new(cik),
        }
    }

    #[must_use]
    pub const fn cik(&self) -> &String {
        self.validated_cik.value()
    }
}

impl StateData for ValidateCikFormatOutputData {
    type UpdateType = ValidateCikFormatOutputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.cik {
            self.validated_cik = Cik::new(&cik);
        }
    }
}

impl Default for ValidateCikFormatOutputData {
    fn default() -> Self {
        Self {
            validated_cik: Cik::new("1067983"),
        }
    }
}

impl fmt::Display for ValidateCikFormatOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tValid CIK: {}", self.validated_cik,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormatOutputDataUpdater {
    pub cik: Option<Cik>,
}

pub struct ValidateCikFormatOutputDataUpdaterBuilder {
    cik: Option<Cik>,
}
impl ValidateCikFormatOutputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { cik: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: &(impl ToString + ?Sized)) -> Self {
        self.cik = Some(Cik::new(cik));
        self
    }

    #[must_use]
    pub fn build(self) -> ValidateCikFormatOutputDataUpdater {
        ValidateCikFormatOutputDataUpdater { cik: self.cik }
    }
}

impl Default for ValidateCikFormatOutputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{ValidateCikFormatOutputData, ValidateCikFormatOutputDataUpdaterBuilder};
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::*;

    #[test]
    fn should_return_reference_to_default_validation_state_data_when_initialized_with_default() {
        let validation_state_data = &ValidateCikFormatOutputData::default();

        let expected_result = &ValidateCikFormatOutputData::default();

        let result = validation_state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let validation_state_data = &ValidateCikFormatOutputData::new("12345");

        let default_validation_state_data = &ValidateCikFormatOutputData::default();

        let result = validation_state_data.get_state();

        assert_ne!(result, default_validation_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_string_when_update_contains_specified_string() {
        let mut state_data = ValidateCikFormatOutputData::default();
        let update = ValidateCikFormatOutputDataUpdaterBuilder::default()
            .cik("12345")
            .build();

        let expected_result = &ValidateCikFormatOutputData::new("12345");

        state_data.update_state(update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut state_data = ValidateCikFormatOutputData::default();
        let update = ValidateCikFormatOutputDataUpdaterBuilder::default()
            .cik("12345")
            .cik("67890")
            .build();

        let expected_result = &ValidateCikFormatOutputData::new("67890");

        state_data.update_state(update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = ValidateCikFormatOutputData::default();
        let empty_update = ValidateCikFormatOutputDataUpdaterBuilder::default().build();

        let expected_result = &ValidateCikFormatOutputData::default();

        state_data.update_state(empty_update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_cik_string_when_validation_output_data_initialized_with_default() {
        let validation_state_data = &ValidateCikFormatOutputData::default();

        let expected_result = "0001067983";

        let result = validation_state_data.get_state().cik();

        assert_eq!(result, expected_result);
    }
    #[test]
    #[should_panic]
    fn should_panic_when_given_invalid_cik_string() {
        let _result = ValidateCikFormatOutputData::new("1234567890a");
    }
}
