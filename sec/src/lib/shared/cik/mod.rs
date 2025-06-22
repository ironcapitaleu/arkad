//! # Central Index Key (CIK) Utilities
//!
//! This module provides the [`Cik`] type and related utilities for parsing and validating the format of
//! SEC Central Index Keys (CIKs). It is used throughout the SEC state machine library to ensure that
//! CIKs are handled in a consistent and robust manner.
//!
//! ## Modules
//! - [`cik_error`]: Error types and reasons for invalid CIKs.
//! - [`constants`]: Constants related to CIK formatting, such as [`CIK_LENGTH`].
//!
//! ## Types
//! - [`Cik`]: Strongly-typed wrapper for a validated CIK string, with constructors and validation logic.
//! - [`CikError`], [`InvalidCikReason`]: Error types for reporting CIK validation failures.
//!
//! ## Usage
//! The [`Cik`] type is used by state input/output data structures (such as in the `validate_cik_format` state)
//! to ensure that only valid CIKs are accepted and processed. The module provides methods for constructing
//! and validating CIKs, as well as error types for detailed error reporting in state transitions and validation routines.
//!
//! ## See Also
//! - [`crate::shared`]: Shared domain types and utilities used across the SEC state machine library.
//! - [`crate::implementations::states::extract::validate_cik_format::vcf_data`]: Modules that use [`Cik`] for input/output validation.
//! - [`crate::error`]: Error types that may reference [`CikError`] and [`InvalidCikReason`] for detailed diagnostics.

pub mod cik_error;
pub mod constants;

pub use cik_error::{CikError, InvalidCikReason};
pub use constants::CIK_LENGTH;

use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Strongly-typed wrapper for a validated SEC Central Index Key (CIK).
///
/// The `Cik` type ensures that only valid, 10-digit, zero-padded numeric CIKs are constructed and used
/// throughout the SEC state machine library. Use [`Cik::new`] to construct and validate a CIK value.
pub struct Cik {
    value: String,
}

impl Cik {
    /// Creates a new [`Cik`] from any value implementing [`ToString`].
    ///
    /// Accepts string slices, `String`, or any type that can be converted to a string.
    /// Trims leading and trailing whitespace and pads with zeros if the input is less than 10 digits.
    ///
    /// # Arguments
    ///
    /// * `cik` - Any type that implements [`ToString`] (e.g., `&str`, `String`, numeric types).
    ///
    /// # Returns
    ///
    /// Returns `Ok(Cik)` if the input is a valid [`Cik`], or a [`CikError`] if the input is invalid.
    ///
    /// # Errors
    ///
    /// Returns a [`CikError`] if the CIK is not formatted correctly (i.e., if the input contains non-numeric characters or exceeds the maximum allowed length).
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::cik::Cik;
    ///
    /// let cik = Cik::new("123456789").expect("CIK creation with the hardcoded value should always succeed.");
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

    /// Validates if a passed CIK contains exactly `CIK_LENGTH` digits.
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
    use super::{CikError, InvalidCikReason};
    use pretty_assertions::{assert_eq, assert_ne};

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
