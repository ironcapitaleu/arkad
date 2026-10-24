//! # SHA-256
//!
//! Provides [`sha256_hex`], which computes the digest that [`DocumentMetadata::sha256`] holds, and
//! the [`Sha256Digest`] newtype that holds such a digest. Callers outside `storage` use the same
//! function, so the digest of a response matches the digest in its metadata file.
//!
//! [`DocumentMetadata::sha256`]: crate::implementations::repository::filesystem::DocumentMetadata::sha256

use std::fmt::{self, Display, Formatter, Write};

use sha2::{Digest, Sha256};

/// Returns the SHA-256 digest of the bytes as 64 lowercase hex characters.
///
/// # Examples
///
/// ```rust
/// use storage::shared::sha256_hex;
///
/// let expected_result = "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a";
///
/// let result = sha256_hex(b"{}");
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
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
/// [`Sha256Digest::of`] is the only constructor, so every value is a digest that [`sha256_hex`]
/// returned.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256Digest(String);

impl Sha256Digest {
    /// Computes the SHA-256 digest of the bytes.
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
    pub fn of(bytes: &[u8]) -> Self {
        Self(sha256_hex(bytes))
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
