use state_maschine::prelude::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Input data for validating the format of a CIK.
pub struct ValidateCikFormatInputData {
    /// The unvalidated CIK string provided for validation.
    pub raw_cik: String,
}

impl ValidateCikFormatInputData {
    /// Creates a new instance of the input data for the CIK format validation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::sec_state_machine::extract::validate_cik_format::vcf_data::vcf_input_data::ValidateCikFormatInputData;
    ///
    /// let validation_input_data = ValidateCikFormatInputData::new("1067983");
    /// ```
    pub fn new(cik: &(impl ToString + ?Sized)) -> Self {
        Self {
            raw_cik: cik.to_string(),
        }
    }

    /// Returns the CIK.
    #[must_use]
    pub const fn cik(&self) -> &String {
        &self.raw_cik
    }
}

impl StateData for ValidateCikFormatInputData {
    type UpdateType = ValidateCikFormatInputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.raw_cik {
            self.raw_cik = cik;
        }
    }
}

const BERKSHIRE_HATHAWAY_CIK: &str = "1067983";

impl Default for ValidateCikFormatInputData {
    /// Returns a default input using the CIK for Berkshire Hathaway (CIK: 1067983).
    fn default() -> Self {
        Self {
            raw_cik: BERKSHIRE_HATHAWAY_CIK.to_string(),
        }
    }
}

impl fmt::Display for ValidateCikFormatInputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tCIK: {}", self.raw_cik,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the validation input.
pub struct ValidateCikFormatInputDataUpdater {
    pub raw_cik: Option<String>,
}

/// Updater builder for the validation input.
pub struct ValidateCikFormatInputDataUpdaterBuilder {
    raw_cik: Option<String>,
}
impl ValidateCikFormatInputDataUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { raw_cik: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: &(impl ToString + ?Sized)) -> Self {
        self.raw_cik = Some(cik.to_string());
        self
    }

    #[must_use]
    pub fn build(self) -> ValidateCikFormatInputDataUpdater {
        ValidateCikFormatInputDataUpdater {
            raw_cik: self.raw_cik,
        }
    }
}

impl Default for ValidateCikFormatInputDataUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use crate::sec_state_machine::extract::validate_cik_format::vcf_data::vcf_input_data::BERKSHIRE_HATHAWAY_CIK;

    use super::{ValidateCikFormatInputData, ValidateCikFormatInputDataUpdaterBuilder};
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::*;

    #[test]
    fn should_return_reference_to_default_validation_state_data_when_initialized_with_default() {
        let default_validation_state_data = ValidateCikFormatInputData::default();

        let expected_result = &ValidateCikFormatInputData::default();

        let result = default_validation_state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let validation_state_data = &ValidateCikFormatInputData::new("0000000000");

        let default_validation_state_data = &ValidateCikFormatInputData::default();

        let result = validation_state_data.get_state();

        assert_ne!(result, default_validation_state_data);
    }

    #[test]
    fn should_update_state_data_to_specified_string_when_update_contains_specified_string() {
        let mut state_data = ValidateCikFormatInputData::default();
        let update = ValidateCikFormatInputDataUpdaterBuilder::default()
            .cik("12345")
            .build();

        let expected_result = &ValidateCikFormatInputData::new("12345");

        state_data.update_state(update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut state_data = ValidateCikFormatInputData::default();
        let update = ValidateCikFormatInputDataUpdaterBuilder::default()
            .cik("1234567890")
            .cik("0000000000")
            .build();

        let expected_result = &ValidateCikFormatInputData::new("0000000000");

        state_data.update_state(update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = ValidateCikFormatInputData::default();
        let empty_update = ValidateCikFormatInputDataUpdaterBuilder::default().build();

        let expected_result = &ValidateCikFormatInputData::default();

        state_data.update_state(empty_update);
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_cik_when_validation_input_data_initialized_with_default() {
        let validation_state_data = &ValidateCikFormatInputData::default();

        let expected_result = &BERKSHIRE_HATHAWAY_CIK.to_string();

        let result = validation_state_data.get_state().cik();

        assert_eq!(result, expected_result);
    }
}
