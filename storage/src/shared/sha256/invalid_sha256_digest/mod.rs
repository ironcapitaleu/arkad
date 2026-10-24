//! # Invalid SHA-256 Digest
//!
//! Provides [`InvalidSha256Digest`], the error raised when a string is not a SHA-256 digest.

use thiserror::Error;

/// Error raised when a string is not 64 lowercase hex characters.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidSha256Digest] SHA-256 digest '{value}' is not 64 lowercase hex characters")]
pub struct InvalidSha256Digest {
    /// The string that was rejected.
    pub value: String,
}

impl InvalidSha256Digest {
    /// Creates a new [`InvalidSha256Digest`] for the rejected string.
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
    const fn should_implement_auto_traits_for_invalid_sha256_digest() {
        implements_auto_traits::<InvalidSha256Digest>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_sha256_digest() {
        implements_value_traits::<InvalidSha256Digest>();
    }

    #[test]
    fn should_format_display_as_expected_for_an_invalid_sha256_digest() {
        let error = InvalidSha256Digest::new("ABC");

        let expected_result =
            "[InvalidSha256Digest] SHA-256 digest 'ABC' is not 64 lowercase hex characters";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
