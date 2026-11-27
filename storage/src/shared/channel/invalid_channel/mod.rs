//! # Invalid Channel
//!
//! Provides [`InvalidChannel`], the error raised when a string is not a channel.

use thiserror::Error;

/// Error raised when a string is not `api_key` or `login`, so it cannot be a channel.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidChannel] Channel '{value}' is not 'api_key' or 'login'")]
pub struct InvalidChannel {
    /// The string that was rejected.
    pub value: String,
}

impl InvalidChannel {
    /// Creates a new [`InvalidChannel`] for the rejected string.
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
    const fn should_implement_auto_traits_for_invalid_channel() {
        implements_auto_traits::<InvalidChannel>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_channel() {
        implements_value_traits::<InvalidChannel>();
    }

    #[test]
    fn should_name_the_rejected_string_when_displaying_invalid_channel() {
        let error = InvalidChannel::new("web");

        let expected_result = "[InvalidChannel] Channel 'web' is not 'api_key' or 'login'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
