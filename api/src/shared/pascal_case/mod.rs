//! # Pascal Case
//!
//! Provides the [`PascalCase`] name and its conversion to `snake_case`.
//!
//! ## Modules
//!
//! - [`invalid_pascal_case`]: The [`InvalidPascalCase`] error for a name that fails the check.

pub mod invalid_pascal_case;

pub use invalid_pascal_case::InvalidPascalCase;

/// A name in `PascalCase`, for example `NetIncome`.
///
/// The name starts with an ASCII uppercase letter and holds only ASCII letters and digits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PascalCase(String);

impl PascalCase {
    /// Creates a new [`PascalCase`] from `name`.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidPascalCase`] if `name` does not start with an ASCII uppercase letter,
    /// or if it holds a character that is not an ASCII letter or digit.
    pub fn new(name: &str) -> Result<Self, InvalidPascalCase> {
        let starts_upper = name.starts_with(|character: char| character.is_ascii_uppercase());
        let alphanumeric = name
            .chars()
            .all(|character| character.is_ascii_alphanumeric());
        if starts_upper && alphanumeric {
            Ok(Self(name.to_owned()))
        } else {
            Err(InvalidPascalCase::new(name))
        }
    }

    /// Returns the name in `snake_case`, for example `net_income` for `NetIncome`.
    ///
    /// Each uppercase letter starts a new word, so `EPSBasic` becomes `e_p_s_basic`.
    #[must_use]
    pub fn to_snake_case(&self) -> String {
        let mut text = String::with_capacity(self.0.len() + 4);
        for (index, character) in self.0.char_indices() {
            if index > 0 && character.is_ascii_uppercase() {
                text.push('_');
            }
            text.push(character.to_ascii_lowercase());
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_pascal_case() {
        implements_auto_traits::<PascalCase>();
    }

    #[test]
    fn should_convert_to_snake_case_when_name_has_two_words() {
        let name = PascalCase::new("NetIncome").expect(
            "Given the hardcoded PascalCase name 'NetIncome', the check should always pass",
        );

        let expected_result = "net_income";

        let result = name.to_snake_case();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_a_single_word_when_converting_assets_to_snake_case() {
        let name = PascalCase::new("Assets")
            .expect("Given the hardcoded PascalCase name 'Assets', the check should always pass");

        let expected_result = "assets";

        let result = name.to_snake_case();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_a_digit_when_name_holds_one() {
        let name = "Q1Revenue";

        let expected_result = Ok("q1_revenue".to_owned());

        let result = PascalCase::new(name).map(|name| name.to_snake_case());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_pascal_case_when_name_starts_lowercase() {
        let name = "netIncome";

        let expected_result = Err(InvalidPascalCase::new("netIncome"));

        let result = PascalCase::new(name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_pascal_case_when_name_holds_a_space() {
        let name = "Net Income";

        let expected_result = Err(InvalidPascalCase::new("Net Income"));

        let result = PascalCase::new(name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_invalid_pascal_case_when_name_is_empty() {
        let name = "";

        let expected_result = Err(InvalidPascalCase::new(""));

        let result = PascalCase::new(name);

        assert_eq!(result, expected_result);
    }
}
