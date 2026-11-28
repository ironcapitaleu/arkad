//! # CRC32C
//!
//! Provides the [`Crc32c`] newtype, the CRC32C checksum of the bytes of a stored object.
//!
//! ## Modules
//!
//! - [`invalid_crc32c`]: The [`InvalidCrc32c`] for a string that is not a CRC32C.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

pub mod invalid_crc32c;

pub use invalid_crc32c::InvalidCrc32c;

/// The CRC32C (Castagnoli) checksum of a byte slice.
///
/// Prints as the base64 of its 4 big-endian bytes, for example `4waSgw==`. [`str::parse`] accepts
/// only that form, so a parsed checksum prints as the text it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Crc32c(u32);

impl Crc32c {
    /// Computes the CRC32C of the bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::shared::Crc32c;
    ///
    /// let bytes = b"123456789";
    ///
    /// let expected_result = 0xE306_9283;
    ///
    /// let result = Crc32c::of(bytes).value();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn of(bytes: impl AsRef<[u8]>) -> Self {
        Self(crc32c::crc32c(bytes.as_ref()))
    }

    /// Returns the checksum as a number.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl Display for Crc32c {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&STANDARD.encode(self.0.to_be_bytes()))
    }
}

impl FromStr for Crc32c {
    type Err = InvalidCrc32c;

    /// Parses the base64 of 4 big-endian bytes into a [`Crc32c`].
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidCrc32c`] if the string is not the padded base64 of exactly 4 bytes, or if
    /// its unused bits are set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::shared::Crc32c;
    ///
    /// let text = "4waSgw==";
    ///
    /// let expected_result = Crc32c::of(b"123456789");
    ///
    /// let result: Crc32c = text
    ///     .parse()
    ///     .expect("Given the CRC32C of '123456789', parsing should always succeed");
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        // `STANDARD` requires the padding and rejects set unused bits, so each value has one text.
        let bytes: [u8; 4] = STANDARD
            .decode(value)
            .ok()
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or_else(|| InvalidCrc32c::new(value))?;
        Ok(Self(u32::from_be_bytes(bytes)))
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
    const fn should_implement_auto_traits_for_crc32c() {
        implements_auto_traits::<Crc32c>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_crc32c() {
        implements_value_traits::<Crc32c>();
    }

    #[test]
    fn should_print_the_base64_of_the_big_endian_bytes_for_the_check_input() {
        let crc32c = Crc32c::of(b"123456789");

        // The CRC32C check value of "123456789" is 0xE3069283.
        let expected_result = "4waSgw==";

        let result = crc32c.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_zero_for_no_bytes() {
        let crc32c = Crc32c::of(b"");

        let expected_result = "AAAAAA==";

        let result = crc32c.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_text_when_it_is_not_base64() {
        let text = "4waSgw!=";

        let expected_result = Err(InvalidCrc32c::new("4waSgw!="));

        let result = text.parse::<Crc32c>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_text_when_it_holds_more_than_4_bytes() {
        let text = "4waSg5M=";

        let expected_result = Err(InvalidCrc32c::new("4waSg5M="));

        let result = text.parse::<Crc32c>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_text_when_it_has_no_padding() {
        let text = "4waSgw";

        let expected_result = Err(InvalidCrc32c::new("4waSgw"));

        let result = text.parse::<Crc32c>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_text_when_its_padding_is_partial() {
        let text = "4waSgw=";

        let expected_result = Err(InvalidCrc32c::new("4waSgw="));

        let result = text.parse::<Crc32c>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_text_when_its_unused_bits_are_set() {
        let text = "4waSgx==";

        let expected_result = Err(InvalidCrc32c::new("4waSgx=="));

        let result = text.parse::<Crc32c>();

        assert_eq!(result, expected_result);
    }
}
