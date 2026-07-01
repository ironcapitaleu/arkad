//! # Central Index Key (CIK)
//!
//! Provides the [`Cik`] type for parsing and validating SEC Central Index Keys.
//!
//! The SEC identifies every filer by a CIK that must be exactly ten zero-padded digits. This
//! module encapsulates that invariant in a newtype, so once a [`Cik`] is constructed the rest of
//! the library can rely on its format without re-validating raw strings.
//!
//! ## Modules
//!
//! - [`cik_error`]: The [`CikError`] returned when validation fails, and its [`InvalidCikReason`].
//! - [`constants`]: Formatting constants such as [`CIK_LENGTH`].
//!
//! ## See Also
//!
//! - [`crate::implementations::states::extract::validate_cik_format`]: The state that validates raw input into a [`Cik`].

pub mod cik_error;
pub mod constants;

pub use cik_error::{CikError, InvalidCikReason};
pub use constants::CIK_LENGTH;

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
/// A validated SEC Central Index Key: exactly ten zero-padded digits.
///
/// Wrapping the CIK in a newtype turns "this string is a well-formed CIK" into a type-level
/// guarantee, so code holding a `Cik` never has to re-validate the format. The inner value is
/// private; values are only produced through [`Cik::new`] (or the [`TryFrom`] impls).
pub struct Cik {
    value: String,
}

impl Cik {
    /// Validates and normalizes a string into a [`Cik`].
    ///
    /// Trims surrounding whitespace and left-pads with zeros to [`CIK_LENGTH`] digits.
    ///
    /// # Errors
    ///
    /// Returns [`CikError`] if the input contains non-numeric characters
    /// ([`InvalidCikReason::ContainsNonNumericCharacters`]) or, after trimming, exceeds
    /// [`CIK_LENGTH`] digits ([`InvalidCikReason::MaxLengthExceeded`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::cik::Cik;
    ///
    /// let cik = Cik::new("123456789").expect("A hardcoded valid CIK should always parse");
    /// assert_eq!(cik.value(), "0123456789");
    /// ```
    pub fn new(cik: &(impl ToString + ?Sized)) -> Result<Self, CikError> {
        let original_input = cik.to_string(); // Keep original input for error messages
        let mut cik_str = cik.to_string().trim().to_string(); // Trim leading and trailing whitespace

        // Check if it contains only digits
        if !cik_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(CikError {
                invalid_cik: original_input,
                reason: InvalidCikReason::ContainsNonNumericCharacters,
            });
        }

        // Prepend zeros if shorter than `CIK_LENGTH` digits
        if cik_str.len() < CIK_LENGTH {
            cik_str = format!("{cik_str:0>CIK_LENGTH$}"); // Pads with leading zeros to a length of `CIK_LENGTH`
        }

        // Ensure the length does not exceed `CIK_LENGTH` digits
        if cik_str.len() > CIK_LENGTH {
            return Err(CikError {
                invalid_cik: original_input,
                reason: InvalidCikReason::MaxLengthExceeded {
                    cik_length: cik_str.len(),
                },
            });
        }

        Ok(Self { value: cik_str })
    }

    /// Returns the underlying CIK string.
    #[must_use]
    pub const fn value(&self) -> &String {
        &self.value
    }

    /// Returns `true` if the string is already exactly [`CIK_LENGTH`] digits, without normalizing.
    #[must_use]
    pub fn is_valid(cik: &str) -> bool {
        cik.len() == CIK_LENGTH && cik.chars().all(|c| c.is_ascii_digit())
    }
}

impl Display for Cik {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TryFrom<&str> for Cik {
    type Error = CikError;

    /// Delegates to [`Cik::new`].
    ///
    /// # Errors
    ///
    /// Returns [`CikError`] if the input is not a valid CIK format.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Cik {
    type Error = CikError;

    /// Delegates to [`Cik::new`].
    ///
    /// # Errors
    ///
    /// Returns [`CikError`] if the input is not a valid CIK format.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::Cik;
    use super::{CikError, InvalidCikReason};
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn should_create_valid_cik_struct_if_numeric_string_with_ten_digits_is_passed() {
        let cik_str = "1234567890";

        let expected_result = "1234567890";

        let result = Cik::new(cik_str)
            .expect("CIK creation should always succeed with hardcoded ten digit value");

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    fn should_fail_when_given_cik_str_that_contains_non_numeric_chars() {
        let cik_str = "12345abcde";

        let result = Cik::new(cik_str);

        assert!(
            result.is_err(),
            "CIK creation with non-numeric chars in hardcoded value should fail."
        );
    }

    #[test]
    fn should_fail_when_given_cik_str_that_is_longer_than_ten_chars_after_trimming() {
        let cik_str = "12345678901";

        let result = Cik::new(cik_str);
        assert!(
            result.is_err(),
            "CIK creation with more than 10 chars after trimming in hardcoded value should fail."
        );
    }

    #[test]
    fn should_fail_when_given_cik_str_that_is_longer_than_ten_chars_and_contains_a_alpha_char() {
        let cik_str = "1234567890a";

        let result = Cik::new(cik_str);
        assert!(
            result.is_err(),
            "CIK creation with more than 10 chars and containing a letter in hardcoded value should fail."
        );
    }

    #[test]
    fn should_prepend_cik_with_zeros_when_passed_valid_string_with_less_than_ten_digits() {
        let cik_str = "123456789";

        let expected_result = "0123456789";

        let result = Cik::new(cik_str).expect("Hardcoded CIK creation should succeed");

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    fn should_create_cik_with_all_zeros_when_passed_empty_string() {
        let cik_str = "";

        let expected_result = "0000000000";

        let result =
            Cik::new(cik_str).expect("Hardcoded CIK creation from empty string should succeed");

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    fn should_trim_whitespace_from_cik_input_str_when_passed_string_with_leading_whitespace() {
        let cik_str = "     0123456789";

        let expected_result = "0123456789";

        let result =
            Cik::new(cik_str).expect("Hardcoded CIK creation with whitespace should succeed");

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    fn should_trim_whitespace_from_cik_input_str_when_passed_string_with_trailing_whitespace() {
        let cik_str = "0123456789     ";

        let expected_result = "0123456789";

        let result =
            Cik::new(cik_str).expect("Hardcoded CIK creation with whitespace should succeed");

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    fn should_trim_whitespace_from_cik_input_str_when_passed_string_with_leading_or_trailing_whitespace()
     {
        let cik_str = "     0123456789     ";

        let expected_result = "0123456789";

        let result =
            Cik::new(cik_str).expect("Hardcoded CIK creation with whitespace should succeed");

        assert_eq!(result.value(), expected_result);
    }

    #[test]
    fn should_return_error_with_reason_non_numeric_when_input_contains_letters_but_has_adequate_length()
     {
        let cik = "12345abcde";

        let expected_result = Err(CikError {
            invalid_cik: cik.to_string(),
            reason: InvalidCikReason::ContainsNonNumericCharacters,
        });

        let result = Cik::new(cik);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_with_reason_non_numeric_when_input_contains_letters_and_is_too_long() {
        let cik = "12345abcde12345";

        let expected_result = Err(CikError {
            invalid_cik: cik.to_string(),
            reason: InvalidCikReason::ContainsNonNumericCharacters,
        });

        let result = Cik::new(cik);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_return_error_with_reason_too_long_when_input_contains_letters_and_is_too_long() {
        let cik = "12345abcde12345";

        let expected_result = Err(CikError {
            invalid_cik: cik.to_string(),
            reason: InvalidCikReason::MaxLengthExceeded {
                cik_length: cik.len(),
            },
        });

        let result = Cik::new(cik);

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_return_error_with_reason_too_long_when_input_does_not_contain_letters_but_is_too_long()
     {
        let cik = "123456789012345";

        let expected_result = Err(CikError {
            invalid_cik: cik.to_string(),
            reason: InvalidCikReason::MaxLengthExceeded {
                cik_length: cik.len(),
            },
        });

        let result = Cik::new(cik);

        assert_eq!(result, expected_result);
    }
}
