use std::fmt;

use crate::sec_state_machine::sec_error::SecError;
use crate::sec_state_machine::sec_state_data::SecStateData;
use state_maschine::prelude::*;

pub mod cik;
pub use cik::Cik;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Output data containing a validated CIK.
pub struct ValidateCikFormatOutputData {
    /// The validated CIK.
    pub validated_cik: Cik,
}

impl ValidateCikFormatOutputData {
    /// Creates a new instance of the output data for the CIK validation state.
    /// The output must follow the correct formatting.
    pub fn new(cik: &(impl ToString + ?Sized)) -> Result<Self, SecError> {
        match Cik::new(cik) {
            Ok(valid_cik) => Ok(Self {
                validated_cik: valid_cik,
            }),
            Err(_) => Err(SecError::InvalidCikFormat(format!(
                "CIK {} is not formatted correctly.",
                cik.to_string()
            ))),
        }
    }

    // Returns the validated CIK.
    #[must_use]
    pub const fn cik(&self) -> &String {
        self.validated_cik.value()
    }
}
impl SecStateData for ValidateCikFormatOutputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), SecError> {
        if let Some(cik) = updates.cik {
            match Cik::new(&cik) {
                Ok(valid_cik) => {
                    self.validated_cik = valid_cik;
                    Ok(())
                }
                Err(_) => Err(SecError::InvalidCikFormat(format!(
                    "CIK {} is not formatted correctly.",
                    cik.value()
                ))),
            }
        } else {
            Ok(())
        }
    }
}
impl StateData for ValidateCikFormatOutputData {
    type UpdateType = ValidateCikFormatOutputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }
    /// Provided by SecStateData trait.
    fn update_state(&mut self, _updates: Self::UpdateType) {
        // This method is not used in this context.
    }
}
const BERKSHIRE_HATHAWAY_CIK: &str = "1067983";

impl Default for ValidateCikFormatOutputData {
    /// Returns a default output using the CIK for Berkshire Hathaway (CIK: 1067983).
    fn default() -> Self {
        Self {
            validated_cik: Cik::new(BERKSHIRE_HATHAWAY_CIK)
                .expect("Hardcoded CIK should always be valid."),
        }
    }
}

impl fmt::Display for ValidateCikFormatOutputData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tValid CIK: {}", self.validated_cik,)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the validation output.
pub struct ValidateCikFormatOutputDataUpdater {
    pub cik: Option<Cik>,
}

/// Updater builder for the validation output.
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
        self.cik = Some(Cik::new(cik).expect("CIK must be valid and formatted correctly"));
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
    use crate::sec_state_machine::extract::validate_cik_format::vcf_data::vcf_output_data::BERKSHIRE_HATHAWAY_CIK;
    use crate::sec_state_machine::sec_error::SecError;
    use crate::sec_state_machine::sec_state_data::SecStateData;

    use super::{Cik, ValidateCikFormatOutputData, ValidateCikFormatOutputDataUpdaterBuilder};
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::*;

    #[test]
    fn should_return_reference_to_default_validation_state_data_when_initialized_with_default() {
        let validation_state_data = ValidateCikFormatOutputData::default();

        let expected_result = &ValidateCikFormatOutputData::default();

        let result = validation_state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let validation_state_data =
            &ValidateCikFormatOutputData::new("12345").expect("CIK must be valid");

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

        let expected_result =
            &ValidateCikFormatOutputData::new("0000012345").expect("CIK must be valid");

        SecStateData::update_state(&mut state_data, update).expect("Update should succeed");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_state_data_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut state_data = ValidateCikFormatOutputData::default();
        let update = ValidateCikFormatOutputDataUpdaterBuilder::default()
            .cik("12345")
            .cik("067890")
            .build();

        let expected_result =
            &ValidateCikFormatOutputData::new("0067890").expect("CIK must be valid");

        SecStateData::update_state(&mut state_data, update).expect("Update should succeed");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = ValidateCikFormatOutputData::default();
        let empty_update = ValidateCikFormatOutputDataUpdaterBuilder::default().build();

        let expected_result = &ValidateCikFormatOutputData::default();

        SecStateData::update_state(&mut state_data, empty_update).expect("Update should succeed");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_formatted_and_validated_default_cik_string_when_validation_output_data_initialized_with_default()
     {
        let validation_state_data = &ValidateCikFormatOutputData::default();
        let formatted_and_validated_berkshire_cik =
            Cik::new(BERKSHIRE_HATHAWAY_CIK).expect("CIK must be valid and formatted correctly");

        let expected_result = formatted_and_validated_berkshire_cik.value();

        let result = validation_state_data.get_state().cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic]
    fn should_panic_when_comparing_valid_but_unformatted_default_cik_with_formatted_and_validated_default_output()
     {
        let validation_state_data = &ValidateCikFormatOutputData::default();
        let valid_but_unformatted_default_cik = BERKSHIRE_HATHAWAY_CIK;

        let result = validation_state_data.get_state().cik();

        assert_eq!(result, valid_but_unformatted_default_cik);
    }

    #[test]
    fn should_fail_when_given_invalid_cik_string() {
        let result = ValidateCikFormatOutputData::new("1234567890a");
        let expected_result = Err(SecError::InvalidCikFormat(
            "CIK 1234567890a is not formatted correctly.".to_string(),
        ));
        assert_eq!(result, expected_result);
    }
}
