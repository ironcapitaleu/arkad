//! # SHA-256
//!
//! Provides the [`Sha256Digest`] newtype that [`DocumentMetadata::sha256`] holds. Callers outside
//! `storage` use the same type, so the digest of a response matches the digest in its metadata
//! file.
//!
//! ## Modules
//!
//! - [`invalid_sha256_digest`]: The [`InvalidSha256Digest`] for a string that is not a digest.
//!
//! [`DocumentMetadata::sha256`]: crate::implementations::repository::filesystem::DocumentMetadata::sha256

use std::fmt::{self, Display, Formatter, Write};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

pub mod invalid_sha256_digest;

pub use invalid_sha256_digest::InvalidSha256Digest;

/// Returns the SHA-256 digest of the bytes as 64 lowercase hex characters.
///
/// Private, so every caller gets a [`Sha256Digest`] through [`Sha256Digest::of`].
fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut hex, byte| {
            // Deviation: the result is ignored. Writing to a `String` never fails.
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

/// The SHA-256 digest of a byte slice, as 64 lowercase hex characters.
///
/// [`Sha256Digest::of`] computes a digest. [`str::parse`] and deserialization accept only 64
/// lowercase hex characters, so every value has the form that [`Sha256Digest::of`] produces. A
/// digest serializes as its hex string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256Digest(String);

impl Sha256Digest {
    /// Computes the SHA-256 digest of the bytes.
    ///
    /// Takes any byte container, such as `&[u8]`, `Vec<u8>`, a byte array, or a `String`. A
    /// `String` gives the digest of its UTF-8 bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::shared::Sha256Digest;
    ///
    /// let expected_result = "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a";
    ///
    /// let digest = Sha256Digest::of(b"{}");
    /// let result = digest.as_str();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn of(bytes: impl AsRef<[u8]>) -> Self {
        Self(sha256_hex(bytes.as_ref()))
    }

    /// Returns the digest as 64 lowercase hex characters.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Sha256Digest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Sha256Digest {
    type Err = InvalidSha256Digest;

    /// Parses 64 lowercase hex characters into a [`Sha256Digest`].
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidSha256Digest`] if the string has another length, or holds a character
    /// that is not `0`–`9` or `a`–`f`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::shared::Sha256Digest;
    ///
    /// let expected_result = Sha256Digest::of(b"{}");
    ///
    /// let result: Sha256Digest = "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
    ///     .parse()
    ///     .expect("Given the digest of '{}', parsing should always succeed");
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let is_lowercase_hex = value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if value.len() != 64 || !is_lowercase_hex {
            return Err(InvalidSha256Digest::new(value));
        }
        Ok(Self(value.to_owned()))
    }
}

impl Serialize for Sha256Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Sha256Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
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
    const fn should_implement_auto_traits_for_sha256_digest() {
        implements_auto_traits::<Sha256Digest>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_sha256_digest() {
        implements_value_traits::<Sha256Digest>();
    }

    const DIGEST_OF_ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn should_parse_the_digest_when_the_string_is_64_lowercase_hex_characters() {
        let expected_result = Ok(Sha256Digest::of(b"abc"));

        let result = DIGEST_OF_ABC.parse::<Sha256Digest>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_sha256_digest_when_the_string_has_63_characters() {
        let short = &DIGEST_OF_ABC[..63];

        let expected_result = Err(InvalidSha256Digest::new(short));

        let result = short.parse::<Sha256Digest>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_sha256_digest_when_the_string_is_uppercase() {
        let uppercase = DIGEST_OF_ABC.to_uppercase();

        let expected_result = Err(InvalidSha256Digest::new(uppercase.clone()));

        let result = uppercase.parse::<Sha256Digest>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_sha256_digest_when_the_string_holds_a_non_hex_character() {
        let non_hex = format!("{}g", &DIGEST_OF_ABC[..63]);

        let expected_result = Err(InvalidSha256Digest::new(non_hex.clone()));

        let result = non_hex.parse::<Sha256Digest>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_round_trip_the_digest_through_json_unchanged() {
        let digest = Sha256Digest::of(b"abc");
        let json = serde_json::to_string(&digest)
            .expect("Given a digest of one string field, serialization should always succeed");

        let expected_result = digest;

        let result: Sha256Digest = serde_json::from_str(&json)
            .expect("Given JSON that a digest serialized, deserialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_digest_as_its_hex_string() {
        let expected_result = format!("\"{DIGEST_OF_ABC}\"");

        let result = serde_json::to_string(&Sha256Digest::of(b"abc"))
            .expect("Given a digest of one string field, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_an_uppercase_digest_when_deserializing() {
        let json = format!("\"{}\"", DIGEST_OF_ABC.to_uppercase());

        let expected_result = true;

        let result = serde_json::from_str::<Sha256Digest>(&json)
            .is_err_and(|error| error.to_string().starts_with("[InvalidSha256Digest]"));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_one_digest_for_a_vec_and_a_slice_of_the_same_bytes() {
        let owned: Vec<u8> = Vec::from("abc");

        let expected_result = Sha256Digest::of(b"abc");

        let result = Sha256Digest::of(owned);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_one_digest_for_a_string_and_its_bytes() {
        let text = String::from("abc");

        let expected_result = Sha256Digest::of(b"abc");

        let result = Sha256Digest::of(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_known_digest_for_the_bytes_abc() {
        let expected_result = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

        let result = Sha256Digest::of(b"abc").to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_known_digest_for_empty_bytes() {
        let expected_result = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

        let result = sha256_hex(b"");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_known_digest_for_the_bytes_abc() {
        let expected_result = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

        let result = sha256_hex(b"abc");

        assert_eq!(result, expected_result);
    }
}
