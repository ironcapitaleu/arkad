use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::error::state_machine::state::InvalidCikFormat;

use crate::traits::state_machine::state::StateData;

use crate::shared::cik::Cik;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Output data containing a validated CIK.
pub struct ValidateCikFormatOutputData {
    /// The validated CIK.
    pub validated_cik: Cik,
}

impl ValidateCikFormatOutputData {
    /// Creates a new instance of the output data for the CIK validation state.
    /// The output must follow the correct formatting.
    ///
    /// # Errors
    ///
    /// Returns a `InvalidCikFormat` if the CIK is not formatted correctly.
    pub fn new(cik: &(impl ToString + ?Sized)) -> Result<Self, StateError> {
        Cik::new(cik).map_or_else(
            |_| {
                Err(StateError::InvalidCikFormat(InvalidCikFormat {
                    reason: format!("CIK '{}' is not formatted correctly.", cik.to_string()),
                    invalid_cik: cik.to_string(),
                    state_name: "ValidateCikFormatOutputData".to_string(),
                }))
            },
            |valid_cik| {
                Ok(Self {
                    validated_cik: valid_cik,
                })
            },
        )
    }

    // Returns the validated CIK.
    #[must_use]
    pub const fn cik(&self) -> &String {
        self.validated_cik.value()
    }
}
impl StateData for ValidateCikFormatOutputData {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(cik) = updates.cik {
            match Cik::new(&cik) {
                Ok(valid_cik) => {
                    self.validated_cik = valid_cik;
                    Ok(())
                }
                Err(_) => Err(StateError::InvalidCikFormat(InvalidCikFormat {
                    reason: format!("CIK '{}' is not formatted correctly.", cik.to_string()),
                    invalid_cik: cik.to_string(),
                    state_name: "ValidateCikFormatOutputData".to_string(),
                })),
            }
        } else {
            Ok(())
        }
    }
}
impl SMStateData for ValidateCikFormatOutputData {
    type UpdateType = ValidateCikFormatOutputDataUpdater;

    fn get_state(&self) -> &Self {
        self
    }
    /// Provided by `SecStateData` trait.
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
    /// Sets the CIK for the updater.
    ///
    /// # Panics
    ///
    /// Panics if the CIK is not valid.
    pub fn cik(mut self, cik: &(impl ToString + ?Sized)) -> Self {
        self.cik = Some(Cik::new(cik).expect("CIK must be valid."));
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
    use crate::implementations::states::extract::validate_cik_format::vcf_data::vcf_output_data::BERKSHIRE_HATHAWAY_CIK;
    use crate::traits::state_machine::state::StateData;

    use super::{Cik, ValidateCikFormatOutputData, ValidateCikFormatOutputDataUpdaterBuilder};
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::StateData as SMStateData;

    #[test]
    fn should_return_reference_to_default_validation_state_data_when_initialized_with_default() {
        let validation_state_data = ValidateCikFormatOutputData::default();

        let expected_result = &ValidateCikFormatOutputData::default();

        let result = validation_state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_state_data_with_custom_data_when_using_new_as_constructor() {
        let validation_state_data = &ValidateCikFormatOutputData::new("12345")
            .expect("Provided hardcoded CIK should always be valid");

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

        let expected_result = &ValidateCikFormatOutputData::new("0000012345")
            .expect("Provided hardcoded CIK should always be valid");

        StateData::update_state(&mut state_data, update)
            .expect("Provided hardcoded update should succeed.");
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

        let expected_result = &ValidateCikFormatOutputData::new("0067890")
            .expect("Provided hardcoded CIK should always be valid.");

        StateData::update_state(&mut state_data, update)
            .expect("Provided hardcoded update should succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = ValidateCikFormatOutputData::default();
        let empty_update = ValidateCikFormatOutputDataUpdaterBuilder::default().build();

        let expected_result = &ValidateCikFormatOutputData::default();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Provided hardcoded update should succeed.");
        let result = state_data.get_state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_formatted_and_validated_default_cik_string_when_validation_output_data_initialized_with_default()
     {
        let validation_state_data = &ValidateCikFormatOutputData::default();
        let formatted_and_validated_berkshire_cik = Cik::new(BERKSHIRE_HATHAWAY_CIK)
            .expect("Provided hardcoded CIK should always be valid.");

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

    // #[test]
    // fn should_fail_when_given_invalid_cik_string() {
    //     let invalid_cik = "1234567890a";

    //     let expected_result: State = StateInvalidCikFormat {
    //         reason: format!("CIK '{}' is not formatted correctly.", invalid_cik),
    //         invalid_cik: invalid_cik.to_string(),
    //         state_name: "ValidateCikFormatOutputData".to_string()
    //         .into(),
    //     };

    //     let result = ValidateCikFormatOutputData::new(invalid_cik).unwrap_err();
    //     assert_eq!(result, expected_result);
    // }
}
