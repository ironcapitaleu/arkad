//! # API Key Name
//!
//! Provides [`ApiKeyName`], the label an account gives one of its API keys.
//!
//! ## Modules
//!
//! - [`invalid_api_key_name`]: The [`InvalidApiKeyName`] for a string that is not a name.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

pub mod invalid_api_key_name;

pub use invalid_api_key_name::InvalidApiKeyName;

/// The most characters an [`ApiKeyName`] holds.
pub const MAX_API_KEY_NAME_CHARACTERS: usize = 100;

/// The label of one API key, such as `CI pipeline`: 1 to 100 characters, without control
/// characters.
///
/// The account area lists the keys by this name. [`ApiKeyName::new`] counts Unicode code points,
/// as `char_length` in Postgres does. In this type, a control character is one of these:
///
/// - A character of the Unicode category `Cc`, such as NUL or a line feed. Postgres `text` cannot
///   hold a NUL character.
/// - The line separator U+2028 and the paragraph separator U+2029.
/// - A character with the Unicode property `Bidi_Control`, such as U+202E. It can show a name
///   in reverse, so one key looks like another.
///
/// A label has no use for any of them. Other format characters stay, such as the zero-width
/// joiner in an emoji.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApiKeyName(String);

impl ApiKeyName {
    /// Creates an [`ApiKeyName`] from 1 to 100 characters without control characters.
    ///
    /// The name stays as given. It is not trimmed.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidApiKeyName`] if the value is empty, holds more than 100 characters, or
    /// holds a control character as [`ApiKeyName`] defines it, such as a line break.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::{ApiKeyName, InvalidApiKeyName};
    ///
    /// let value = "";
    ///
    /// let expected_result = Err(InvalidApiKeyName::new(""));
    ///
    /// let result = ApiKeyName::new(value);
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidApiKeyName> {
        let value = value.into();
        let character_count = value.chars().count();
        let has_control_character = value.chars().any(is_control_character);
        if !(1..=MAX_API_KEY_NAME_CHARACTERS).contains(&character_count) || has_control_character {
            return Err(InvalidApiKeyName { value });
        }
        Ok(Self(value))
    }

    /// Returns the name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ApiKeyName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ApiKeyName {
    type Err = InvalidApiKeyName;

    /// Parses a name with the rules of [`ApiKeyName::new`].
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidApiKeyName`] in the cases that [`ApiKeyName::new`] lists.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// Returns `true` for a control character as [`ApiKeyName`] defines it.
const fn is_control_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{2028}'
                | '\u{2029}'
                | '\u{061C}'
                | '\u{200E}'
                | '\u{200F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2066}'..='\u{2069}'
        )
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_api_key_name() {
        implements_auto_traits::<ApiKeyName>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_api_key_name() {
        implements_value_traits::<ApiKeyName>();
    }

    #[test]
    fn should_create_the_api_key_name_when_the_string_has_one_character() {
        let value = "a";

        let expected_result = Ok("a".to_owned());

        let result = ApiKeyName::new(value).map(|name| name.as_str().to_owned());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_the_api_key_name_when_the_string_has_100_characters() {
        let value = "a".repeat(100);

        let expected_result = Ok(value.clone());

        let result = ApiKeyName::new(value).map(|name| name.as_str().to_owned());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_the_api_key_name_when_100_characters_take_more_than_100_bytes() {
        let value = "ä".repeat(100);

        let expected_result = Ok(value.clone());

        let result = ApiKeyName::new(value).map(|name| name.as_str().to_owned());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_spaces_around_the_name_when_creating_the_api_key_name() {
        let value = " CI pipeline ";

        let expected_result = Ok(" CI pipeline ".to_owned());

        let result = ApiKeyName::new(value).map(|name| name.as_str().to_owned());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_api_key_name_when_the_string_is_empty() {
        let value = "";

        let expected_result = Err(InvalidApiKeyName::new(""));

        let result = ApiKeyName::new(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_api_key_name_when_the_string_has_101_characters() {
        let value = "a".repeat(101);

        let expected_result = Err(InvalidApiKeyName::new(value.clone()));

        let result = ApiKeyName::new(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_api_key_name_when_the_string_holds_a_nul_character() {
        let value = "CI\0pipeline";

        let expected_result = Err(InvalidApiKeyName::new("CI\0pipeline"));

        let result = ApiKeyName::new(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_api_key_name_when_the_string_holds_a_line_break() {
        let value = "CI\npipeline";

        let expected_result = Err(InvalidApiKeyName::new("CI\npipeline"));

        let result = ApiKeyName::new(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_name_for_an_api_key_name() {
        let api_key_name =
            ApiKeyName::new("CI pipeline").expect("Given a short name without control characters, the creation of an API key name should always succeed");

        let expected_result = "CI pipeline";

        let result = api_key_name.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_the_displayed_text_back_to_the_same_api_key_name() {
        let api_key_name =
            ApiKeyName::new("Prod – Zürich").expect("Given a short name without control characters, the creation of an API key name should always succeed");
        let text = api_key_name.to_string();

        let expected_result = Ok(api_key_name);

        let result = text.parse::<ApiKeyName>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_api_key_name_when_the_string_holds_a_line_separator() {
        let value = "CI\u{2028}pipeline";

        let expected_result = Err(InvalidApiKeyName::new("CI\u{2028}pipeline"));

        let result = ApiKeyName::new(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_api_key_name_when_the_string_holds_a_right_to_left_override() {
        let value = "\u{202E}enilepip IC";

        let expected_result = Err(InvalidApiKeyName::new("\u{202E}enilepip IC"));

        let result = ApiKeyName::new(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_the_api_key_name_when_the_string_holds_an_emoji_with_a_zero_width_joiner() {
        let value = "Dev \u{1F468}\u{200D}\u{1F4BB}";

        let expected_result = Ok("Dev \u{1F468}\u{200D}\u{1F4BB}".to_owned());

        let result = ApiKeyName::new(value).map(|name| name.as_str().to_owned());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_each_end_of_the_separator_and_bidi_control_ranges_for_an_api_key_name() {
        let characters = [
            '\u{2028}', '\u{2029}', '\u{061C}', '\u{200E}', '\u{200F}', '\u{202A}', '\u{202E}',
            '\u{2066}', '\u{2069}',
        ];

        let expected_result: Vec<char> = vec![];

        let result: Vec<char> = characters
            .into_iter()
            .filter(|character| ApiKeyName::new(character.to_string()).is_ok())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_each_neighbor_of_the_separator_and_bidi_control_ranges_for_an_api_key_name() {
        let characters = [
            '\u{2027}', '\u{202F}', '\u{061B}', '\u{061D}', '\u{200D}', '\u{2065}', '\u{206A}',
        ];

        let expected_result: Vec<char> = vec![];

        let result: Vec<char> = characters
            .into_iter()
            .filter(|character| ApiKeyName::new(character.to_string()).is_err())
            .collect();

        assert_eq!(result, expected_result);
    }
}
