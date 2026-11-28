//! # API Key
//!
//! Provides [`ApiKey`], the secret that a client sends to call `/v1` without a login.
//!
//! ## Modules
//!
//! - [`failed_api_key_generation`]: The [`FailedApiKeyGeneration`] for a read of the random source
//!   that fails.
//! - [`invalid_api_key`]: The [`InvalidApiKey`] for a string that is not an API key.

use std::fmt::{self, Debug, Display, Formatter};

use storage::{KeyEnvironment, Sha256Digest};
use zeroize::Zeroize;

pub mod failed_api_key_generation;
pub mod invalid_api_key;

pub use failed_api_key_generation::FailedApiKeyGeneration;
pub use invalid_api_key::InvalidApiKey;

/// The digits of base62, in ASCII order.
const BASE62_ALPHABET: &[u8; 62] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// The number of random bytes in the secret.
const SECRET_BYTES: usize = 32;

/// The length of `ak_live_` and `ak_test_`.
const PREFIX_LENGTH: usize = 8;

/// The base62 digits of the secret: the fewest digits that hold every 32-byte value.
const SECRET_LENGTH: usize = 43;

/// The base62 digits of the checksum: the fewest digits that hold every CRC32 value.
const CHECKSUM_LENGTH: usize = 6;

/// The index of the `_` between the secret and the checksum.
const SEPARATOR_INDEX: usize = PREFIX_LENGTH + SECRET_LENGTH;

/// The length of a full key: prefix, secret, separator, and checksum.
const KEY_LENGTH: usize = SEPARATOR_INDEX + 1 + CHECKSUM_LENGTH;

/// The length of the display prefix: the prefix and the first 4 characters of the secret.
const DISPLAY_PREFIX_LENGTH: usize = PREFIX_LENGTH + 4;

/// An API key, such as `ak_live_<43 base62 characters>_<6 base62 characters>`.
///
/// The key has four parts:
///
/// - The prefix `ak_live_` or `ak_test_` names the [`KeyEnvironment`].
/// - The secret is 32 random bytes as one big-endian number in base62 (`0-9A-Za-z`), padded with
///   `0` to 43 characters.
/// - A `_` separates the secret from the checksum.
/// - The checksum is the CRC32 of the prefix and the secret, in base62, padded to 6 characters.
///
/// The checksum lets [`ApiKey::parse`] reject a mistyped or invented key without a database call.
/// It covers the prefix, so a key of one environment does not parse as the other.
///
/// The API stores only the [`ApiKey::digest`]. [`Debug`] and [`Display`] print the
/// [`ApiKey::display_prefix`], never the secret. Only [`ApiKey::expose_secret`] returns the full
/// key. A drop overwrites the key in memory with zeros.
#[derive(Clone)]
pub struct ApiKey {
    environment: KeyEnvironment,
    value: String,
}

impl ApiKey {
    /// Generates a new [`ApiKey`] from 32 bytes of the random source of the operating system.
    ///
    /// # Errors
    ///
    /// Returns a [`FailedApiKeyGeneration`] if the operating system cannot give random bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use api::shared::api_key::ApiKey;
    /// use storage::KeyEnvironment;
    ///
    /// let environment = KeyEnvironment::Test;
    ///
    /// let expected_result = "ak_test_";
    ///
    /// let api_key = ApiKey::generate(environment)
    ///     .expect("Given a Linux host with a random source, key generation should always succeed");
    /// let result = &api_key.display_prefix()[..8];
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn generate(environment: KeyEnvironment) -> Result<Self, FailedApiKeyGeneration> {
        let mut secret = [0_u8; SECRET_BYTES];
        getrandom::fill(&mut secret)
            .map_err(|error| FailedApiKeyGeneration::new(error.to_string()))?;
        let api_key = Self::from_secret(environment, &secret);
        secret.zeroize();
        Ok(api_key)
    }

    /// Parses a key and checks its prefix, its length, its characters, and its checksum.
    ///
    /// The check reads no database and does no I/O. A key that passes can still be unknown or
    /// revoked. The check does not reject a secret of 2²⁵⁶ or more, which no generated key holds.
    /// Such a key fails at the digest lookup.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidApiKey`] for the first check that fails, in this order:
    ///
    /// - [`InvalidApiKey::InvalidPrefix`] if the key does not start with `ak_live_` or `ak_test_`.
    /// - [`InvalidApiKey::InvalidLength`] if the key does not have 58 bytes.
    /// - [`InvalidApiKey::InvalidCharacter`] if the secret or the checksum holds a byte that is
    ///   not base62, or if the byte between them is not `_`.
    /// - [`InvalidApiKey::InvalidChecksum`] if the checksum does not match the prefix and the
    ///   secret.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use api::shared::api_key::{ApiKey, InvalidApiKey};
    ///
    /// let value = "ak_live_003aUlTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf_0gEjkX";
    ///
    /// let expected_result = Some(InvalidApiKey::InvalidChecksum);
    ///
    /// let result = ApiKey::parse(value).err();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn parse(value: &str) -> Result<Self, InvalidApiKey> {
        let environment = [KeyEnvironment::Live, KeyEnvironment::Test]
            .into_iter()
            .find(|environment| value.starts_with(&prefix_of(*environment)))
            .ok_or(InvalidApiKey::InvalidPrefix)?;
        if value.len() != KEY_LENGTH {
            return Err(InvalidApiKey::InvalidLength {
                length: value.len(),
            });
        }
        if let Some(index) = value
            .bytes()
            .enumerate()
            .skip(PREFIX_LENGTH)
            .find_map(|(index, byte)| (!fits_format_at(index, byte)).then_some(index))
        {
            return Err(InvalidApiKey::InvalidCharacter { index });
        }
        let (body, checksum) = (&value[..SEPARATOR_INDEX], &value[SEPARATOR_INDEX + 1..]);
        if checksum != checksum_of(body) {
            return Err(InvalidApiKey::InvalidChecksum);
        }
        Ok(Self {
            environment,
            value: value.to_owned(),
        })
    }

    /// Returns the environment that the prefix names.
    #[must_use]
    pub const fn environment(&self) -> KeyEnvironment {
        self.environment
    }

    /// Returns the SHA-256 digest of the full key, the only form of the key that the API stores.
    #[must_use]
    pub fn digest(&self) -> Sha256Digest {
        Sha256Digest::of(self.value.as_bytes())
    }

    /// Returns the prefix and the first 4 characters of the secret, such as `ak_live_3f9c`.
    ///
    /// The account area shows it, so a person can tell the keys apart.
    #[must_use]
    pub fn display_prefix(&self) -> &str {
        &self.value[..DISPLAY_PREFIX_LENGTH]
    }

    /// Returns the full key, with its secret.
    ///
    /// Only the response that creates the key shows it, once. Never log the result.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.value
    }

    /// Builds the key of `environment` from the bytes of a secret.
    fn from_secret(environment: KeyEnvironment, secret: &[u8; SECRET_BYTES]) -> Self {
        // One allocation of the full length, so no outgrown buffer with the secret stays unwiped.
        let mut value = String::with_capacity(KEY_LENGTH);
        value.push_str(&prefix_of(environment));
        let mut secret_digits = base62_of_bytes(secret, SECRET_LENGTH);
        value.push_str(&secret_digits);
        secret_digits.zeroize();
        let checksum = checksum_of(&value);
        value.push('_');
        value.push_str(&checksum);
        Self { environment, value }
    }
}

impl Drop for ApiKey {
    /// Overwrites the key with zeros, so the freed memory holds no secret.
    fn drop(&mut self) {
        self.value.zeroize();
    }
}

impl Debug for ApiKey {
    /// Writes the environment and the display prefix, never the secret.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiKey")
            .field("environment", &self.environment)
            .field("display_prefix", &self.display_prefix())
            .finish_non_exhaustive()
    }
}

impl Display for ApiKey {
    /// Writes the display prefix and an ellipsis, such as `ak_live_3f9c…`, never the secret.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}…", self.display_prefix())
    }
}

/// Returns the prefix of the environment, such as `ak_live_`.
fn prefix_of(environment: KeyEnvironment) -> String {
    format!("ak_{}_", environment.as_str())
}

/// Returns `true` if `byte` fits the format at `index`: `_` at the separator, base62 elsewhere.
const fn fits_format_at(index: usize, byte: u8) -> bool {
    if index == SEPARATOR_INDEX {
        byte == b'_'
    } else {
        byte.is_ascii_alphanumeric()
    }
}

/// Returns the CRC32 of `body` in base62, padded to 6 characters.
fn checksum_of(body: &str) -> String {
    base62_of_bytes(
        &crc32fast::hash(body.as_bytes()).to_be_bytes(),
        CHECKSUM_LENGTH,
    )
}

/// Returns `bytes`, read as one big-endian number, in base62 with exactly `width` digits.
///
/// Each pass divides the number by 62 and keeps the remainder as the next digit, from the right.
/// The caller picks a `width` that holds every value of `bytes`, so no digit is lost.
fn base62_of_bytes(bytes: &[u8], width: usize) -> String {
    let mut number = bytes.to_vec();
    let mut digits = vec![b'0'; width];
    for digit in digits.iter_mut().rev() {
        let mut remainder = 0_u16;
        for byte in &mut number {
            let value = (remainder << 8) | u16::from(*byte);
            *byte = u8::try_from(value / 62).expect(
                "Given a remainder below 62, the quotient of a two-byte value by 62 should always fit a byte",
            );
            remainder = value % 62;
        }
        *digit = BASE62_ALPHABET[usize::from(remainder)];
    }
    String::from_utf8(digits)
        .expect("Given digits from the ASCII base62 alphabet, the bytes should always be UTF-8")
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    /// The key of the secret bytes 0, 1, …, 31 in the live environment, computed with Python's
    /// `zlib.crc32` and `int.from_bytes`.
    const SEQUENCE_KEY: &str = "ak_live_003aUlTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf_0gEjkH";

    fn sequence_secret() -> [u8; SECRET_BYTES] {
        std::array::from_fn(|index| {
            u8::try_from(index)
                .expect("Given an index below 32, the index should always fit a byte")
        })
    }

    fn parsed(value: &str) -> ApiKey {
        ApiKey::parse(value).expect(
            "Given a key that the reference implementation wrote, parsing should always succeed",
        )
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_api_key() {
        implements_auto_traits::<ApiKey>();
    }

    const fn implements_clone_and_debug<T: Clone + Debug>() {}
    #[test]
    const fn should_implement_clone_and_debug_for_api_key() {
        implements_clone_and_debug::<ApiKey>();
    }

    #[test]
    fn should_write_the_known_key_when_building_from_the_sequence_secret() {
        let secret = sequence_secret();

        let expected_result = SEQUENCE_KEY;

        let api_key = ApiKey::from_secret(KeyEnvironment::Live, &secret);
        let result = api_key.expose_secret();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_known_key_when_building_from_a_secret_of_zero_bytes() {
        let secret = [0_u8; SECRET_BYTES];

        let expected_result = "ak_test_0000000000000000000000000000000000000000000_0JaaOf";

        let api_key = ApiKey::from_secret(KeyEnvironment::Test, &secret);
        let result = api_key.expose_secret();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_known_key_when_building_from_a_secret_of_max_bytes() {
        let secret = [u8::MAX; SECRET_BYTES];

        let expected_result = "ak_live_yhjskwdA6OZ1AL1YmHWZWm8LLG7HjnuCA2j5rOw8Xp1_2zsbnN";

        let api_key = ApiKey::from_secret(KeyEnvironment::Live, &secret);
        let result = api_key.expose_secret();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_crc32_in_six_base62_characters_for_the_sequence_key() {
        let body = &SEQUENCE_KEY[..SEPARATOR_INDEX];

        // CRC32 0x25334b19 in base62.
        let expected_result = "0gEjkH";

        let result = checksum_of(body);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_known_digest_for_the_sequence_key() {
        let api_key = parsed(SEQUENCE_KEY);

        let expected_result = "af687470e764f80fb818b43def833448649b99bb556aab84b8a4ec70e53cdb89";

        let result = api_key.digest().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_prefix_and_four_secret_characters_as_the_display_prefix() {
        let api_key = parsed(SEQUENCE_KEY);

        let expected_result = "ak_live_003a";

        let result = api_key.display_prefix();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_a_generated_key_back_to_the_same_key() {
        let api_key = ApiKey::generate(KeyEnvironment::Live).expect(
            "Given a Linux host with a random source, key generation should always succeed",
        );

        let expected_result = Ok(api_key.expose_secret().to_owned());

        let result =
            ApiKey::parse(api_key.expose_secret()).map(|parsed| parsed.expose_secret().to_owned());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_two_different_keys_when_generating_twice() {
        let first = ApiKey::generate(KeyEnvironment::Live).expect(
            "Given a Linux host with a random source, key generation should always succeed",
        );
        let second = ApiKey::generate(KeyEnvironment::Live).expect(
            "Given a Linux host with a random source, key generation should always succeed",
        );

        let expected_result = first.expose_secret();

        let result = second.expose_secret();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_parse_a_live_key_as_the_live_environment() {
        let value = SEQUENCE_KEY;

        let expected_result = Ok(KeyEnvironment::Live);

        let result = ApiKey::parse(value).map(|api_key| api_key.environment());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_a_test_key_as_the_test_environment() {
        let value = "ak_test_0000000000000000000000000000000000000000000_0JaaOf";

        let expected_result = Ok(KeyEnvironment::Test);

        let result = ApiKey::parse(value).map(|api_key| api_key.environment());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_checksum_when_a_live_key_gets_the_test_prefix() {
        let value = "ak_test_003aUlTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf_0gEjkH";

        let expected_result = Some(InvalidApiKey::InvalidChecksum);

        let result = ApiKey::parse(value).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_prefix_when_the_key_has_an_unknown_environment() {
        let value = "ak_prod_003aUlTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf_0gEjkH";

        let expected_result = Some(InvalidApiKey::InvalidPrefix);

        let result = ApiKey::parse(value).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_prefix_when_the_value_is_a_login_token() {
        let value = "eyJhbGciOiJFUzI1NiJ9.eyJzdWIiOiIxIn0.c2lnbmF0dXJl";

        let expected_result = Some(InvalidApiKey::InvalidPrefix);

        let result = ApiKey::parse(value).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_length_when_the_key_misses_a_character() {
        let value = "ak_live_003aUlTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf_0gEjk";

        let expected_result = Some(InvalidApiKey::InvalidLength { length: 57 });

        let result = ApiKey::parse(value).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_length_when_the_key_has_a_trailing_line_break() {
        let value = "ak_live_003aUlTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf_0gEjkH\n";

        let expected_result = Some(InvalidApiKey::InvalidLength { length: 59 });

        let result = ApiKey::parse(value).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_character_when_the_secret_holds_a_hyphen() {
        let value = "ak_live_003a-lTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf_0gEjkH";

        let expected_result = Some(InvalidApiKey::InvalidCharacter { index: 12 });

        let result = ApiKey::parse(value).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_character_when_the_separator_is_not_an_underscore() {
        let value = "ak_live_003aUlTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf-0gEjkH";

        let expected_result = Some(InvalidApiKey::InvalidCharacter { index: 51 });

        let result = ApiKey::parse(value).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_character_when_the_checksum_holds_a_non_ascii_byte() {
        let value = "ak_live_003aUlTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf_0gEjä";

        let expected_result = Some(InvalidApiKey::InvalidCharacter { index: 56 });

        let result = ApiKey::parse(value).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_checksum_when_one_secret_character_changes() {
        let value = "ak_live_003bUlTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf_0gEjkH";

        let expected_result = Some(InvalidApiKey::InvalidChecksum);

        let result = ApiKey::parse(value).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_no_secret_when_formatting_the_key_with_debug() {
        let api_key = parsed(SEQUENCE_KEY);

        let expected_result = "ApiKey { environment: Live, display_prefix: \"ak_live_003a\", .. }";

        let result = format!("{api_key:?}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_no_secret_when_formatting_the_key_with_display() {
        let api_key = parsed(SEQUENCE_KEY);

        let expected_result = "ak_live_003a…";

        let result = api_key.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_character_when_the_first_secret_character_is_a_hyphen() {
        let value = "ak_live_-03aUlTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf_0gEjkH";

        let expected_result = Some(InvalidApiKey::InvalidCharacter { index: 8 });

        let result = ApiKey::parse(value).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_character_when_the_last_checksum_character_is_a_hyphen() {
        let value = "ak_live_003aUlTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf_0gEjk-";

        let expected_result = Some(InvalidApiKey::InvalidCharacter { index: 57 });

        let result = ApiKey::parse(value).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_a_secret_above_two_to_the_256_when_its_checksum_matches() {
        // 43 times `z` is 62^43 - 1, above 2^256. Python's `zlib.crc32` gives the checksum.
        let value = "ak_live_zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz_1xmd6O";

        let expected_result = Ok("ak_live_zzzz".to_owned());

        let result = ApiKey::parse(value).map(|api_key| api_key.display_prefix().to_owned());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_the_key_in_one_allocation_of_key_length_when_building_from_a_secret() {
        let secret = sequence_secret();

        let expected_result = KEY_LENGTH;

        let api_key = ApiKey::from_secret(KeyEnvironment::Live, &secret);
        let result = api_key.value.capacity();

        assert_eq!(result, expected_result);
    }
}
