//! # MIC
//!
//! Provides [`Mic`], the Market Identifier Code of ISO 10383 that names an exchange.
//!
//! ## Modules
//!
//! - [`invalid_mic`]: The [`InvalidMic`] for a string that is not a MIC.

use std::fmt::{self, Display, Formatter};

pub mod invalid_mic;

pub use invalid_mic::InvalidMic;

/// The Market Identifier Code of ISO 10383, such as `XNAS`: four upper-case ASCII letters or
/// digits.
///
/// A listing names its exchange by its MIC. [`Mic::new`] checks the form only. It does not
/// check that the ISO 10383 list holds the code.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mic(String);

impl Mic {
    /// Creates a [`Mic`] from four upper-case ASCII letters or digits.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidMic`] if the value has another length, or holds a character that is
    /// not `A`–`Z` or `0`–`9`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::{InvalidMic, Mic};
    ///
    /// let value = "xnas";
    ///
    /// let expected_result = Err(InvalidMic::new("xnas"));
    ///
    /// let result = Mic::new(value);
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidMic> {
        let value = value.into();
        let is_upper_alphanumeric = value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit());
        if value.len() != 4 || !is_upper_alphanumeric {
            return Err(InvalidMic { value });
        }
        Ok(Self(value))
    }

    /// Returns the MIC as four characters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::Mic;
    ///
    /// let value = "XNAS";
    ///
    /// let expected_result = "XNAS";
    ///
    /// let mic = Mic::new(value)
    ///     .expect("Given four upper-case letters, the creation of a MIC should always succeed");
    /// let result = mic.as_str();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Mic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_mic() {
        implements_auto_traits::<Mic>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_mic() {
        implements_value_traits::<Mic>();
    }

    #[test]
    fn should_create_the_mic_when_the_string_is_four_upper_case_letters() {
        let mic_string = "XNAS";

        let expected_result = Ok("XNAS".to_owned());

        let result = Mic::new(mic_string).map(|mic| mic.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_the_mic_when_the_string_holds_a_digit() {
        let mic_string = "C2OX";

        let expected_result = Ok("C2OX".to_owned());

        let result = Mic::new(mic_string).map(|mic| mic.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_mic_when_the_string_holds_a_lower_case_letter() {
        let mic_string = "XNAs";

        let expected_result = Err(InvalidMic::new("XNAs"));

        let result = Mic::new(mic_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_mic_when_the_string_has_three_characters() {
        let mic_string = "XNA";

        let expected_result = Err(InvalidMic::new("XNA"));

        let result = Mic::new(mic_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_mic_when_the_string_has_five_characters() {
        let mic_string = "XNASD";

        let expected_result = Err(InvalidMic::new("XNASD"));

        let result = Mic::new(mic_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_mic_when_the_string_is_empty() {
        let mic_string = "";

        let expected_result = Err(InvalidMic::new(""));

        let result = Mic::new(mic_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_mic_when_the_string_holds_a_punctuation_character() {
        let mic_string = "XN-S";

        let expected_result = Err(InvalidMic::new("XN-S"));

        let result = Mic::new(mic_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_mic_when_the_string_has_four_bytes_but_a_non_ascii_character() {
        let mic_string = "XNÄ";

        let expected_result = Err(InvalidMic::new("XNÄ"));

        let result = Mic::new(mic_string);

        assert_eq!(result, expected_result);
    }
}
