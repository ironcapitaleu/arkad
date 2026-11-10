//! # Release
//!
//! Provides [`TaxonomyRelease`], the taxonomy release that the tables of a plug-in come from.

use std::fmt;
use std::str::FromStr;

use super::fact::TaxonomyVersion;
use super::invalid_value::{InvalidValue, InvalidValueReason, require_token};

/// The ID of a taxonomy release: the family and the year, for example `us-gaap/2026`. A release
/// ID has the form of a [`TaxonomyVersion`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct TaxonomyReleaseId(String);

impl TaxonomyReleaseId {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "TaxonomyReleaseId";

    /// The character between the family and the year, as in `us-gaap/2026`.
    pub const SEPARATOR: char = TaxonomyVersion::SEPARATOR;

    /// The number of digits of the year.
    pub const YEAR_LENGTH: usize = TaxonomyVersion::YEAR_LENGTH;

    /// Creates a [`TaxonomyReleaseId`] from its text.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the text is empty, holds whitespace, holds no `/`, has an empty
    /// family, or has a year that is not four digits.
    pub fn new(text: impl Into<String>) -> Result<Self, InvalidValue> {
        let text = text.into();
        require_token(Self::TYPE_NAME, &text)?;
        let Some((family, year)) = text.split_once(Self::SEPARATOR) else {
            return Err(InvalidValue::new(
                Self::TYPE_NAME,
                InvalidValueReason::MissingSeparator(Self::SEPARATOR),
                text,
            ));
        };
        if family.is_empty() {
            return Err(InvalidValue::new(
                Self::TYPE_NAME,
                InvalidValueReason::Empty,
                text,
            ));
        }
        if year.len() != Self::YEAR_LENGTH || !year.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(InvalidValue::new(
                Self::TYPE_NAME,
                InvalidValueReason::NotANumber,
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
pub struct Sha256([u8; Self::BYTE_LENGTH]);

impl Sha256 {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "Sha256";

    /// The number of bytes of a digest.
    pub const BYTE_LENGTH: usize = 32;

    /// The number of hex digits that write one byte.
    const HEX_DIGITS_PER_BYTE: usize = 2;

    /// The number of hex digits of a digest.
    pub const HEX_LENGTH: usize = Self::HEX_DIGITS_PER_BYTE * Self::BYTE_LENGTH;

    /// The radix of the hex digits.
    const HEX_RADIX: u32 = 16;

    /// Creates a [`Sha256`] from the 32 bytes of a digest.
    #[must_use]
    pub const fn new(bytes: [u8; Self::BYTE_LENGTH]) -> Self {
        Self(bytes)
    }

    /// Returns the 32 bytes of the digest.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; Self::BYTE_LENGTH] {
        &self.0
    }
}

impl FromStr for Sha256 {
    type Err = InvalidValue;

    /// Parses a [`Sha256`] from 64 hex digits.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] with [`InvalidValueReason::NotHexDigest`] if the text is not 64 hex
    /// digits.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || InvalidValue::new(Self::TYPE_NAME, InvalidValueReason::NotHexDigest, text);
        if text.len() != Self::HEX_LENGTH || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(invalid());
        }
        let mut bytes = [0; Self::BYTE_LENGTH];
        for (byte, pair) in bytes
            .iter_mut()
            .zip(text.as_bytes().chunks(Self::HEX_DIGITS_PER_BYTE))
        {
            let pair = std::str::from_utf8(pair).map_err(|_| invalid())?;
            *byte = u8::from_str_radix(pair, Self::HEX_RADIX).map_err(|_| invalid())?;
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

/// A taxonomy release: its ID and the SHA-256 of the `MANIFEST` files of its vendored folders.
///
/// A standardized fact carries the release of the plug-in that resolved it, so the fact proves its
/// release without a join.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct TaxonomyRelease {
    /// The ID, for example `us-gaap/2026`.
    pub id: TaxonomyReleaseId,
    /// The SHA-256 of the `MANIFEST` files of the vendored folders.
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
            InvalidValueReason::NotHexDigest,
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
            InvalidValueReason::NotHexDigest,
            text,
        ));

        let result = Sha256::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_release_id_when_year_is_empty() {
        let text = "us-gaap/";

        let expected_result = Err(InvalidValue::new(
            "TaxonomyReleaseId",
            InvalidValueReason::NotANumber,
            "us-gaap/",
        ));

        let result = TaxonomyReleaseId::new(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_release_id_when_family_is_empty() {
        let text = "/2026";

        let expected_result = Err(InvalidValue::new(
            "TaxonomyReleaseId",
            InvalidValueReason::Empty,
            "/2026",
        ));

        let result = TaxonomyReleaseId::new(text);

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
