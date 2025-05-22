use crate::state_machine::sec_error::SecError;
use crate::state_machine::sec_state::SecState;
use state_maschine::prelude::*;
use std::fmt;

pub mod vcf_context;
pub mod vcf_data;

pub use vcf_context::ValidateCikFormatContext;
use vcf_data::Cik;
pub use vcf_data::ValidateCikFormatInputData;
pub use vcf_data::ValidateCikFormatOutputData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// State that validates and normalizes a raw CIK format.
///
/// The state takes an unvalidated CIK string as input, checks for format correctness,
/// and ensures the CIK is correctly zero-padded to 10 digits. It does **not** verify
/// the existence of the CIK in the SEC database.
///
/// # Behavior
/// - Trims leading and trailing whitespace.
/// - Ensures the CIK contains only digits.
/// - Prepends leading zeros if necessary.
/// - Produces a validated 'Cik' object containing the normalized CIK.
///
/// # Limitations
/// - This validation is **syntactic only**. It does **not** check whether the CIK actually exists in the SEC records.
///
/// # Output
/// The validated CIK is stored internally after calling `compute_output_data()`.
///
/// # Example
/// ```
/// use sec::state_machine::extract::validate_cik_format::*;
///
/// let input = ValidateCikFormatInputData { raw_cik: "1234".into() };
/// let context = ValidateCikFormatContext::default();
/// let mut validation_state = ValidateCikFormat::new(input, context);
/// ```
pub struct ValidateCikFormat {
    input: ValidateCikFormatInputData,
    context: ValidateCikFormatContext,
    output: Option<ValidateCikFormatOutputData>,
}

impl ValidateCikFormat {
    #[must_use]
    pub const fn new(input: ValidateCikFormatInputData, context: ValidateCikFormatContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }
}

impl SecState for ValidateCikFormat {
    fn compute_output_data(&mut self) -> Result<(), SecError> {
        // Validate the CIK format
        let cik = Cik::new(&self.input.raw_cik);

        match cik {
            Ok(cik) => {
                // If the CIK is valid, set the output data
                self.output = Some(ValidateCikFormatOutputData { validated_cik: cik });
            }
            Err(e) => {
                // If the CIK is invalid, return an error
                return Err(e);
            }
        }

        Ok(())
    }
}

impl State for ValidateCikFormat {
    type InputData = ValidateCikFormatInputData;
    type OutputData = ValidateCikFormatOutputData;
    type Context = ValidateCikFormatContext;

    fn get_state_name(&self) -> impl ToString {
        "CIK Format Validation"
    }

    /// Validates if the given CIK has the correct format.
    /// Does nothing here, as the output data is computed in `compute_output_data() of the 'SecState'-implementation`.
    fn compute_output_data(&mut self) {
        // No action needed here, as the output data is computed in `compute_output_data() of the 'SecState'-implementation`
        // This function is just a placeholder to satisfy the State trait.
    }

    fn get_context_data(&self) -> &Self::Context {
        &self.context
    }

    fn get_input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn get_output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl fmt::Display for ValidateCikFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.get_state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};

    #[test]
    fn should_return_name_of_validation_state_when_in_validation_state() {
        let validation_state = ValidateCikFormat::default();

        let expected_result = String::from("CIK Format Validation");

        let result = validation_state.get_state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_validation_data_struct_as_input_data_when_in_initial_validation_state()
    {
        let validation_state = ValidateCikFormat::default();

        let expected_result = &ValidateCikFormatInputData::default();

        let result = validation_state.get_input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let validation_state = ValidateCikFormat::default();

        let _result = validation_state
            .get_output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let validation_state = ValidateCikFormat::default();

        let expected_result = false;

        let result = validation_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let validation_state = ValidateCikFormat::default();

        let expected_result = &ValidateCikFormatContext::default();

        let result = validation_state.get_context_data();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_traits_when_implementing_state_trait() {
        implements_auto_traits::<ValidateCikFormat>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_trait() {
        implements_send::<ValidateCikFormat>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_trait() {
        implements_sync::<ValidateCikFormat>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_trait() {
        implements_send::<ValidateCikFormat>();
        implements_sync::<ValidateCikFormat>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_trait() {
        implements_sized::<ValidateCikFormat>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_trait() {
        implements_hash::<ValidateCikFormat>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_trait() {
        implements_partial_eq::<ValidateCikFormat>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_trait() {
        implements_eq::<ValidateCikFormat>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_trait() {
        implements_partial_ord::<ValidateCikFormat>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_trait() {
        implements_ord::<ValidateCikFormat>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_state_trait() {
        implements_default::<ValidateCikFormat>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_trait() {
        implements_debug::<ValidateCikFormat>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_trait() {
        implements_clone::<ValidateCikFormat>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_trait() {
        implements_unpin::<ValidateCikFormat>();
    }

    #[test]
    fn should_return_default_context_data_when_called_with_state_reference() {
        let validation_state = &ValidateCikFormat::default();
        let ref_to_validation_state = &ValidateCikFormat::default();

        let expected_result = validation_state.get_context_data();

        let result = ref_to_validation_state.get_context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_validation_state = &mut ValidateCikFormat::default();

        let expected_result = false;

        let result = ref_to_validation_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_reference_state()
     {
        let ref_to_validation_state = &ValidateCikFormat::default();

        let _result = ref_to_validation_state
            .get_output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_name_of_validation_state_when_calling_reference_to_validation_state() {
        let ref_to_validation_state = &ValidateCikFormat::default();

        let expected_result = String::from("CIK Format Validation");

        let result = ref_to_validation_state.get_state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_as_input_data_when_reference_validation_state_in_initial_state()
     {
        let ref_to_validation_state = &ValidateCikFormat::default();

        let expected_result = &ValidateCikFormatInputData::default();

        let result = ref_to_validation_state.get_input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_change_input_data_when_computing_output_data() {
        let mut validation_state = ValidateCikFormat::default();

        let expected_result = &validation_state.get_input_data().clone();

        SecState::compute_output_data(&mut validation_state)
            .expect("Default state should always compute output data.");
        let result = validation_state.get_input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_correct_output_data_when_computing_output_data() {
        let mut validation_state = ValidateCikFormat::default();

        let expected_result = &ValidateCikFormatOutputData::default();

        SecState::compute_output_data(&mut validation_state)
            .expect("Default state should always compute output data.");

        let result = validation_state.get_output_data().unwrap();

        assert_eq!(result, expected_result);
    }
}
