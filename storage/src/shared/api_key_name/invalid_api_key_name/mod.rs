//! # Invalid API Key Name
//!
//! Provides [`InvalidApiKeyName`], the error raised when a string is not an API key name.

use thiserror::Error;

use crate::shared::api_key_name::MAX_API_KEY_NAME_CHARACTERS;

/// Error raised when a string is empty, holds more than 100 characters, or holds a control
/// character as [`ApiKeyName`](crate::shared::ApiKeyName) defines it, so it cannot be an API key
/// name.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[InvalidApiKeyName] API key name '{value}' is not 1 to {max} characters without control characters",
    max = MAX_API_KEY_NAME_CHARACTERS
)]
pub struct InvalidApiKeyName {
    /// The string that was rejected.
    pub value: String,
}

impl InvalidApiKeyName {
    /// Creates a new [`InvalidApiKeyName`] for the rejected string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
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
    const fn should_implement_auto_traits_for_invalid_api_key_name() {
        implements_auto_traits::<InvalidApiKeyName>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_api_key_name() {
        implements_value_traits::<InvalidApiKeyName>();
    }

    #[test]
    fn should_name_the_rejected_string_when_displaying_invalid_api_key_name() {
        let error = InvalidApiKeyName::new("");

        let expected_result = "[InvalidApiKeyName] API key name '' is not 1 to 100 characters without control characters";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
