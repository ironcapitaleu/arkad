//! # Invalid CRC32C
//!
//! Provides [`InvalidCrc32c`], the error raised when a string is not a CRC32C checksum.

use thiserror::Error;

/// Error raised when a string is not the base64 of 4 bytes.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidCrc32c] CRC32C '{value}' is not the base64 of 4 bytes")]
pub struct InvalidCrc32c {
    /// The string that was rejected.
    pub value: String,
}

impl InvalidCrc32c {
    /// Creates a new [`InvalidCrc32c`] for the rejected string.
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
    const fn should_implement_auto_traits_for_invalid_crc32c() {
        implements_auto_traits::<InvalidCrc32c>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_crc32c() {
        implements_value_traits::<InvalidCrc32c>();
    }

    #[test]
    fn should_format_display_as_expected_for_an_invalid_crc32c() {
        let error = InvalidCrc32c::new("abc");

        let expected_result = "[InvalidCrc32c] CRC32C 'abc' is not the base64 of 4 bytes";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
