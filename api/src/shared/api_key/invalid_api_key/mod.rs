//! # Invalid API Key
//!
//! Provides [`InvalidApiKey`], the error raised when a string is not an API key.

use thiserror::Error;

/// Error indicating that a string does not have the format of an API key.
///
/// [`ApiKey::parse`](crate::shared::api_key::ApiKey::parse) raises it without a database call. No
/// variant prints the rejected string, because the string can be a real secret with a typo.
#[non_exhaustive]
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum InvalidApiKey {
    /// The string does not start with `ak_live_` or `ak_test_`.
    #[error("[InvalidPrefix] API key does not start with 'ak_live_' or 'ak_test_'")]
    InvalidPrefix,

    /// The string does not have the 58 bytes of a key.
    #[error("[InvalidLength] API key has {length} bytes, not 58")]
    InvalidLength {
        /// The number of bytes in the string.
        length: usize,
    },

    /// A byte of the secret or the checksum is not base62, or the separator is not `_`.
    #[error(
        "[InvalidCharacter] API key byte at index {index} does not fit 'ak_<environment>_<secret>_<checksum>'"
    )]
    InvalidCharacter {
        /// The index of the first byte that does not fit, counted from 0.
        index: usize,
    },

    /// The checksum does not match the prefix and the secret.
    #[error("[InvalidChecksum] API key checksum is not the CRC32 of the prefix and the secret")]
    InvalidChecksum,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_invalid_api_key() {
        implements_auto_traits::<InvalidApiKey>();
    }

    const fn implements_value_traits<
        T: std::fmt::Debug + Clone + Copy + PartialEq + Eq + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_api_key() {
        implements_value_traits::<InvalidApiKey>();
    }

    #[test]
    fn should_name_both_prefixes_when_displaying_invalid_prefix() {
        let error = InvalidApiKey::InvalidPrefix;

        let expected_result =
            "[InvalidPrefix] API key does not start with 'ak_live_' or 'ak_test_'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_length_when_displaying_invalid_length() {
        let error = InvalidApiKey::InvalidLength { length: 57 };

        let expected_result = "[InvalidLength] API key has 57 bytes, not 58";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_index_when_displaying_invalid_character() {
        let error = InvalidApiKey::InvalidCharacter { index: 51 };

        let expected_result = "[InvalidCharacter] API key byte at index 51 does not fit 'ak_<environment>_<secret>_<checksum>'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_crc32_when_displaying_invalid_checksum() {
        let error = InvalidApiKey::InvalidChecksum;

        let expected_result =
            "[InvalidChecksum] API key checksum is not the CRC32 of the prefix and the secret";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
