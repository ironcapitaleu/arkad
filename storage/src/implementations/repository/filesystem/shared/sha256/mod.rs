//! # SHA-256
//!
//! Provides [`sha256_hex`], which computes the digest that [`DocumentMetadata::sha256`] holds.
//!
//! [`DocumentMetadata::sha256`]: crate::implementations::repository::filesystem::DocumentMetadata::sha256

use std::fmt::Write;

use sha2::{Digest, Sha256};

/// Returns the SHA-256 digest of the bytes as 64 lowercase hex characters.
///
/// # Examples
///
/// ```rust
/// use storage::implementations::repository::filesystem::sha256_hex;
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

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
