//! # Invalid API Key ID
//!
//! Provides [`InvalidApiKeyId`], the error raised when a string is not an API key ID.

use thiserror::Error;

/// Error raised when a string is not a UUID, so it cannot be an API key ID.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidApiKeyId] API key ID '{value}' is not a UUID")]
pub struct InvalidApiKeyId {
    /// The string that was rejected.
    pub value: String,
}

impl InvalidApiKeyId {
    /// Creates a new [`InvalidApiKeyId`] for the rejected string.
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
    const fn should_implement_auto_traits_for_invalid_api_key_id() {
        implements_auto_traits::<InvalidApiKeyId>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_api_key_id() {
        implements_value_traits::<InvalidApiKeyId>();
    }

    #[test]
    fn should_name_the_rejected_string_when_displaying_invalid_api_key_id() {
        let error = InvalidApiKeyId::new("key-42");

        let expected_result = "[InvalidApiKeyId] API key ID 'key-42' is not a UUID";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
