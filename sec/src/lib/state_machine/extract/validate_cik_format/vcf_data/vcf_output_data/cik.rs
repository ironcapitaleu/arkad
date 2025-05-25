#![allow(clippy::missing_const_for_fn)]
use crate::error::State as StateError;
use std::fmt;

const CIK_LENGTH: usize = 10;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct Cik {
    value: String,
}

impl Cik {
    /// Creates a new `Cik` from a string, trimming whitespace and padding with zeros if less than 10 digits.
    ///
    /// # Errors
    ///
    /// Returns a `error::State::InvalidCikFormat` if the CIK is not formatted correctly.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::state_machine::extract::validate_cik_format::vcf_data::vcf_output_data::cik::Cik;
    ///
    /// let cik = Cik::new("123456789").expect("CIK creation with the hardcoded value should always succeed.");
    /// assert_eq!(cik.value(), "0123456789");
    /// ```
    pub fn new(cik: &(impl ToString + ?Sized)) -> Result<Self, StateError> {
        let original_input = cik.to_string(); // Keep original input for error messages
        let mut cik_str = cik.to_string().trim().to_string(); // Trim leading and trailing whitespace

        // Check if it contains only digits

        if !cik_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(StateError::InvalidCikFormat(format!(
                "CIK must contain only numeric characters. Got: '{original_input}'" // Show original input in error
            )));
        }

        // Prepend zeros if shorter than `CIK_LENGTH` digits
        if cik_str.len() < CIK_LENGTH {
            cik_str = format!("{cik_str:0>CIK_LENGTH$}"); // Pads with leading zeros to a length of `CIK_LENGTH`
        }

        // Ensure the length does not exceed `CIK_LENGTH` digits
        if cik_str.len() > CIK_LENGTH {
            return Err(StateError::InvalidCikFormat(format!(
                "Final CIK cannot exceed {CIK_LENGTH} digits. Got: '{original_input}'"
            )));
        }

        Ok(Self { value: cik_str })
    }

    /// Returns the underlying CIK string.
    #[must_use]
    pub const fn value(&self) -> &String {
        &self.value
    }

    /// Validates if the CIK contains exactly `CIK_LENGTH` digits.
    #[must_use]
    pub fn is_valid(cik: &str) -> bool {
        cik.len() == CIK_LENGTH && cik.chars().all(|c| c.is_ascii_digit())
    }
}

impl fmt::Display for Cik {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::Cik;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_valid_cik_struct_if_numeric_string_with_ten_digits_is_passed() {
        let cik_str = "1234567890";

        let expected_result = "1234567890";

        let result = Cik::new(cik_str)
            .expect("CIK creation should always succeed with hardcoded ten digit value.");

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    fn should_fail_when_given_cik_str_that_contains_non_numeric_chars() {
        let result = Cik::new("12345abcd!");
        assert!(
            result.is_err(),
            "CIK creation with non-numeric chars in hardcoded value should fail."
        );
    }

    #[test]
    fn should_fail_when_given_cik_str_that_is_longer_than_ten_chars_after_trimming() {
        let result = Cik::new("12345678901");
        assert!(
            result.is_err(),
            "CIK creation with more than 10 chars after trimming in hardcoded value should fail."
        );
    }

    #[test]
    fn should_fail_when_given_cik_str_that_is_longer_than_ten_chars_and_contains_a_alpha_char() {
        let result = Cik::new("1234567890a");
        assert!(
            result.is_err(),
            "CIK creation with more than 10 chars and containing a letter in hardcoded value should fail."
        );
    }

    #[test]
    fn should_prepend_cik_with_zeros_when_passed_valid_string_with_less_than_ten_digits() {
        let cik_str = "123456789";

        let expected_result = "0123456789";

        let result = Cik::new(cik_str).expect("Hardcoded CIK creation should succeed.");

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    fn should_create_cik_with_all_zeros_when_passed_empty_string() {
        let cik_str = "";

        let expected_result = "0000000000";

        let result =
            Cik::new(cik_str).expect("Hardcoded CIK creation from empty string should succeed.");

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    fn should_trim_whitespace_from_cik_input_str_when_passed_string_with_leading_whitespace() {
        let cik_str = "     0123456789";

        let expected_result = "0123456789";

        let result =
            Cik::new(cik_str).expect("Hardcoded CIK creation with whitespace should succeed.");

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    fn should_trim_whitespace_from_cik_input_str_when_passed_string_with_trailing_whitespace() {
        let cik_str = "0123456789     ";

        let expected_result = "0123456789";

        let result =
            Cik::new(cik_str).expect("Hardcoded CIK creation with whitespace should succeed.");

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    fn should_trim_whitespace_from_cik_input_str_when_passed_string_with_leading_or_trailing_whitespace()
     {
        let cik_str = "     0123456789     ";

        let expected_result = "0123456789";

        let result =
            Cik::new(cik_str).expect("Hardcoded CIK creation with whitespace should succeed.");

        assert_eq!(result.value(), expected_result);
    }
}
