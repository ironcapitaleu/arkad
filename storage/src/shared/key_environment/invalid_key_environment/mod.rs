//! # Invalid Key Environment
//!
//! Provides [`InvalidKeyEnvironment`], the error raised when a string is not a key environment.

use thiserror::Error;

/// Error raised when a string is not `live` or `test`, so it cannot be a key environment.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidKeyEnvironment] Key environment '{value}' is not 'live' or 'test'")]
pub struct InvalidKeyEnvironment {
    /// The string that was rejected.
    pub value: String,
}

impl InvalidKeyEnvironment {
    /// Creates a new [`InvalidKeyEnvironment`] for the rejected string.
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
    const fn should_implement_auto_traits_for_invalid_key_environment() {
        implements_auto_traits::<InvalidKeyEnvironment>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_key_environment() {
        implements_value_traits::<InvalidKeyEnvironment>();
    }

    #[test]
    fn should_name_the_rejected_string_when_displaying_invalid_key_environment() {
        let error = InvalidKeyEnvironment::new("prod");

        let expected_result =
            "[InvalidKeyEnvironment] Key environment 'prod' is not 'live' or 'test'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
