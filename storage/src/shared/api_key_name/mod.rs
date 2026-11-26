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
/// as `char_length` in Postgres does. It rejects control characters, because Postgres `text`
/// cannot hold a NUL character and a label has no use for a line break.
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
    /// holds a control character, such as a line break.
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
        let has_control_character = value.chars().any(char::is_control);
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
}
