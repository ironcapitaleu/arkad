//! # Body Digest
//!
//! Provides the [`BodyDigest`] newtype: a precomputed hash of an HTTP response body.

use std::fmt::{self, Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};

use serde::Serialize;

/// A precomputed `u64` digest of a response body.
///
/// Lets [`SecResponse`](super::SecResponse) and downstream types derive `Hash` and `Ord` cheaply,
/// without re-serializing a large JSON body. The digest is taken over the *raw* body text at
/// construction (before JSON parsing), so it reflects the exact bytes received; the single
/// construction path ([`BodyDigest::from_body_text`]) keeps that computation consistent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct BodyDigest(u64);

impl BodyDigest {
    /// Computes a digest from raw body text.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::response::implementations::sec_response::body_digest::BodyDigest;
    ///
    /// let digest = BodyDigest::from_body_text("some body text");
    /// assert_eq!(digest, BodyDigest::from_body_text("some body text"));
    /// ```
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

impl Display for BodyDigest {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
