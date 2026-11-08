//! # Release
//!
//! Provides [`TaxonomyRelease`], the taxonomy release that the tables of a plug-in come from.

use std::fmt;
use std::str::FromStr;

use super::invalid_value::{InvalidValue, InvalidValueReason, require_token};

/// The ID of a taxonomy release: the family and the year, for example `us-gaap/2026`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct TaxonomyReleaseId(String);

impl TaxonomyReleaseId {
    /// Creates a [`TaxonomyReleaseId`] from its text.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the text is empty, holds whitespace, or holds no `/`.
    pub fn new(text: impl Into<String>) -> Result<Self, InvalidValue> {
        let text = text.into();
        require_token("TaxonomyReleaseId", &text)?;
        if !text.contains('/') {
            return Err(InvalidValue::new(
                "TaxonomyReleaseId",
                InvalidValueReason::MissingSeparator('/'),
                text,
            ));
        }
        Ok(Self(text))
    }

    /// Returns the text of the ID.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TaxonomyReleaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A SHA-256 digest, printed as 64 lowercase hex digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Sha256([u8; 32]);

impl Sha256 {
    /// Creates a [`Sha256`] from the 32 bytes of a digest.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the 32 bytes of the digest.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl FromStr for Sha256 {
    type Err = InvalidValue;

    /// Parses a [`Sha256`] from 64 hex digits.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] with [`InvalidValueReason::NotANumber`] if the text is not 64 hex
    /// digits.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || InvalidValue::new("Sha256", InvalidValueReason::NotANumber, text);
        if text.len() != 64 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(invalid());
        }
        let mut bytes = [0; 32];
        for (byte, pair) in bytes.iter_mut().zip(text.as_bytes().chunks(2)) {
            let pair = std::str::from_utf8(pair).map_err(|_| invalid())?;
            *byte = u8::from_str_radix(pair, 16).map_err(|_| invalid())?;
        }
        Ok(Self(bytes))
    }
}

impl fmt::Display for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// A taxonomy release: its ID and the SHA-256 of the `MANIFEST` of its vendored files.
///
/// A standardized fact carries the release of the plug-in that resolved it, so the fact proves its
/// release without a join.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct TaxonomyRelease {
    /// The ID, for example `us-gaap/2026`.
    pub id: TaxonomyReleaseId,
    /// The SHA-256 of the `MANIFEST` of the vendored files.
    pub checksum: Sha256,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const DIGEST: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_taxonomy_release() {
        implements_auto_traits::<TaxonomyRelease>();
    }

    #[test]
    fn should_print_same_hex_when_digest_is_parsed_and_displayed() {
        let text = DIGEST;

        let expected_result = DIGEST;

        let result = Sha256::from_str(text)
            .expect("Given 64 hardcoded hex digits, the parse should always succeed")
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_digest_when_text_is_too_short() {
        let text = "9f86d0";

        let expected_result = Err(InvalidValue::new(
            "Sha256",
            InvalidValueReason::NotANumber,
            "9f86d0",
        ));

        let result = Sha256::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_digest_when_text_holds_a_non_hex_digit() {
        let text = "zz86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

        let expected_result = Err(InvalidValue::new(
            "Sha256",
            InvalidValueReason::NotANumber,
            text,
        ));

        let result = Sha256::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_release_id_when_it_has_no_year() {
        let text = "us-gaap";

        let expected_result = Err(InvalidValue::new(
            "TaxonomyReleaseId",
            InvalidValueReason::MissingSeparator('/'),
            "us-gaap",
        ));

        let result = TaxonomyReleaseId::new(text);

        assert_eq!(result, expected_result);
    }
}
