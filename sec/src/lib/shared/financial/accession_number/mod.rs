//! # Accession Number
//!
//! Provides the [`AccessionNumber`] newtype for SEC filing accession numbers, and the
//! [`InvalidAccessionNumber`] error that [`AccessionNumber::new`] returns for a malformed input.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;
use thiserror::Error;

/// The length of an accession number such as `0000320193-17-000070`.
const ACCESSION_NUMBER_LENGTH: usize = 20;

/// The byte positions of the two dashes in an accession number.
const DASH_POSITIONS: [usize; 2] = [10, 13];

/// An SEC filing accession number — the unique ID of one EDGAR submission.
///
/// Holds 10 digits, `-`, 2 digits, `-`, and 6 digits, for example `0000320193-17-000070`.
/// [`AccessionNumber::new`] checks this shape, so the folder-index URL built from it names the
/// folder of the filing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct AccessionNumber(String);

impl AccessionNumber {
    /// Creates an [`AccessionNumber`] from a string of the shape `##########-##-######`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidAccessionNumber`] if the input has another length, or if a character is
    /// not a digit where the shape needs a digit, or not `-` where it needs a dash.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::financial::accession_number::AccessionNumber;
    ///
    /// let accn = AccessionNumber::new("0000320193-23-000106")
    ///     .expect("Given a hardcoded accession number of the right shape, creation should always succeed");
    ///
    /// let expected_result = "0000320193-23-000106";
    ///
    /// let result = accn.value();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidAccessionNumber> {
        let value = value.into();
        let has_accession_shape = value.len() == ACCESSION_NUMBER_LENGTH
            && value.bytes().enumerate().all(|(index, byte)| {
                if DASH_POSITIONS.contains(&index) {
                    byte == b'-'
                } else {
                    byte.is_ascii_digit()
                }
            });

        if has_accession_shape {
            Ok(Self(value))
        } else {
            Err(InvalidAccessionNumber {
                invalid_accession_number: value,
            })
        }
    }

    /// Returns a reference to the accession number string.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl Display for AccessionNumber {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Deviation: the `domain-concept` skill pairs a `{Type}Error` struct with an
// `Invalid{Type}Reason` enum. The shape is the only rule here, so that enum would hold one
// variant. A single leaf error says the same in fewer types.
/// Error returned by [`AccessionNumber::new`] for an input that is not of the shape
/// `##########-##-######`.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[InvalidAccessionNumber] Accession number '{invalid_accession_number}' does not have the shape '##########-##-######'"
)]
pub struct InvalidAccessionNumber {
    /// The input that failed the check.
    pub invalid_accession_number: String,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_value_when_accession_number_is_created() {
        let accn = AccessionNumber::new("0000320193-23-000106").expect(
            "Given a hardcoded accession number of the right shape, creation should always succeed",
        );

        let expected_result = "0000320193-23-000106";

        let result = accn.value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_accession_number_when_formatted() {
        let accn = AccessionNumber::new("0000320193-23-000106").expect(
            "Given a hardcoded accession number of the right shape, creation should always succeed",
        );

        let expected_result = "0000320193-23-000106";

        let result = accn.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_accession_number_when_input_has_accession_shape() {
        let input = "0000320193-17-000070";

        let expected_result = Ok(AccessionNumber("0000320193-17-000070".to_string()));

        let result = AccessionNumber::new(input);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_input_is_one_digit_short() {
        let input = "0000320193-17-00007";

        let expected_result = Err(InvalidAccessionNumber {
            invalid_accession_number: "0000320193-17-00007".to_string(),
        });

        let result = AccessionNumber::new(input);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_input_has_a_letter_in_a_digit_position() {
        let input = "0000320193-1A-000070";

        let expected_result = Err(InvalidAccessionNumber {
            invalid_accession_number: "0000320193-1A-000070".to_string(),
        });

        let result = AccessionNumber::new(input);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_input_has_a_digit_in_a_dash_position() {
        let input = "0000320193017-000070";

        let expected_result = Err(InvalidAccessionNumber {
            invalid_accession_number: "0000320193017-000070".to_string(),
        });

        let result = AccessionNumber::new(input);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_error_when_input_has_a_multibyte_character_of_the_right_byte_length() {
        let input = "0000320193-17-0000é";

        let expected_result = Err(InvalidAccessionNumber {
            invalid_accession_number: "0000320193-17-0000é".to_string(),
        });

        let result = AccessionNumber::new(input);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_for_invalid_accession_number() {
        let error = InvalidAccessionNumber {
            invalid_accession_number: "%2e%2e".to_string(),
        };

        let expected_result = "[InvalidAccessionNumber] Accession number '%2e%2e' does not have the shape '##########-##-######'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    const fn should_implement_auto_traits_for_invalid_accession_number() {
        const fn implements_auto_traits<T: Send + Sync + Unpin + Sized>() {}
        implements_auto_traits::<InvalidAccessionNumber>();
    }
}
