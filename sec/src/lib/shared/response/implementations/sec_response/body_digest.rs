//! # Body Digest Module
//!
//! Provides the [`BodyDigest`] newtype for a precomputed hash of an HTTP response body.
//! Computed once at construction time and reused for `Hash` and `Ord` implementations,
//! avoiding expensive re-serialization of large JSON payloads.

use std::fmt;
use std::hash::{DefaultHasher, Hash, Hasher};

/// A precomputed `u64` digest of a response body.
///
/// Computed from body text using [`DefaultHasher`] at construction time.
/// Used by [`SecResponse`](super::SecResponse) and downstream types to implement
/// `Hash` and `Ord` efficiently without re-serializing the JSON body.
///
/// Only one construction path exists ([`from_body_text`](Self::from_body_text)) to ensure
/// digest consistency. The idea is that the digest is computed on the raw body text,
/// before the body is parsed as JSON. This way, the digest reflects the exact content of the body,
/// and the digest computation is more efficient since it does not need to convert the JSON value back to text for hashing.
///
/// # Example
/// ```
/// use sec::shared::response::implementations::sec_response::body_digest::BodyDigest;
///
/// let digest = BodyDigest::from_body_text("some body text");
/// assert_eq!(digest, BodyDigest::from_body_text("some body text"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct BodyDigest(u64);

impl BodyDigest {
    /// Computes a digest from a string slice (typically the raw HTTP response body).
    #[must_use]
    pub fn from_body_text(body_text: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        body_text.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Returns the raw `u64` digest value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for BodyDigest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_produce_same_digest_when_input_is_identical() {
        let expected_result = BodyDigest::from_body_text("hello world");

        let result = BodyDigest::from_body_text("hello world");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_different_digest_when_input_differs() {
        let a = BodyDigest::from_body_text("hello");
        let b = BodyDigest::from_body_text("world");

        let expected_result = false;

        let result = a == b;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_as_hex_when_formatted() {
        let digest = BodyDigest::from_body_text("test");

        let expected_result = 16;

        let result = digest.to_string().len();

        assert_eq!(result, expected_result);
    }
}
