//! # Byte Form
//!
//! Provides [`ByteForm`], the byte form of the values of a silver row that `content_digest` hashes.
//!
//! ## Modules
//!
//! - [`constants`]: The marker bytes of a null value and of a value.

use std::io::Write;

use bigdecimal::BigDecimal;
use bigdecimal::num_bigint::Sign;
use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};

use self::constants::{NULL_MARKER, VALUE_MARKER};

pub mod constants;

/// A byte buffer that holds values in the byte form of `content_digest`.
///
/// The transform code prints each value, and the Arrow crate prints none. So the digest of the same
/// rows stays equal across Arrow releases and integer widths. Each method appends one value. A
/// value starts with a marker byte: `0x00` for null, `0x01` for a value. A scalar then enters as a
/// 4-byte big-endian length and its UTF-8 text. A list or a struct enters as a 4-byte
/// big-endian count, and its elements or fields follow in their own byte form.
///
/// [ADR 0033](https://github.com/ironcapitaleu/arkad/blob/main/docs/adr/0033-byte-form-of-content-digest.md)
/// fixes the form. A change of a print rule changes every digest, and needs a new
/// [`TRANSFORM_VERSION`](crate::shared::transform_version::TRANSFORM_VERSION).
///
/// # Examples
///
/// ```rust
/// use sec::shared::row_digest::ByteForm;
///
/// let concept = "Assets";
///
/// let expected_result = [0x01, 0, 0, 0, 6, b'A', b's', b's', b'e', b't', b's'];
///
/// let mut byte_form = ByteForm::new();
/// byte_form.utf8(concept);
/// let result = byte_form.as_bytes();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteForm {
    bytes: Vec<u8>,
}

impl ByteForm {
    /// Creates an empty [`ByteForm`].
    #[must_use]
    pub const fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    /// Returns the bytes that the values appended so far form.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the bytes and consumes the [`ByteForm`].
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Appends a null value of any type: the marker `0x00` and no more bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sec::shared::row_digest::ByteForm;
    ///
    /// let mut byte_form = ByteForm::new();
    ///
    /// // null: 00
    /// let expected_result: &[u8] = b"\x00";
    ///
    /// let result = byte_form.null().as_bytes();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn null(&mut self) -> &mut Self {
        self.bytes.push(NULL_MARKER);
        self
    }

    /// Appends a `utf8` value as the string as stored.
    ///
    /// # Panics
    ///
    /// Panics if the string is longer than `u32::MAX` bytes, because the length prefix has 4 bytes.
    /// No Arrow `utf8` value is that long, because its offsets are 32-bit signed integers, so a
    /// value holds at most about 2 GiB. A `large_utf8` column has 64-bit offsets and can reach the
    /// panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sec::shared::row_digest::ByteForm;
    ///
    /// let namespace = "us-gaap";
    ///
    /// // marker 01 | length 00000007 | text us-gaap
    /// let expected_result: &[u8] = b"\x01\x00\x00\x00\x07us-gaap";
    ///
    /// let mut byte_form = ByteForm::new();
    /// let result = byte_form.utf8(namespace).as_bytes();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn utf8(&mut self, value: &str) -> &mut Self {
        self.text(|text| text.extend_from_slice(value.as_bytes()))
    }

    /// Appends a `bool` value as `true` or `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sec::shared::row_digest::ByteForm;
    ///
    /// let is_custom = true;
    ///
    /// // marker 01 | length 00000004 | text true
    /// let expected_result: &[u8] = b"\x01\x00\x00\x00\x04true";
    ///
    /// let mut byte_form = ByteForm::new();
    /// let result = byte_form.bool(is_custom).as_bytes();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn bool(&mut self, value: bool) -> &mut Self {
        let text = if value { "true" } else { "false" };
        self.utf8(text)
    }

    /// Appends a value of any Arrow integer type in base 10, with `-` before a negative value.
    ///
    /// Every integer width gives the same text, so a column that widens keeps its digest.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sec::shared::row_digest::ByteForm;
    ///
    /// let quarters: u16 = 4007;
    ///
    /// // marker 01 | length 00000004 | text 4007
    /// let expected_result: &[u8] = b"\x01\x00\x00\x00\x044007";
    ///
    /// let mut byte_form = ByteForm::new();
    /// let result = byte_form.integer(quarters).as_bytes();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn integer(&mut self, value: impl Into<i128>) -> &mut Self {
        let value: i128 = value.into();
        self.text(|text| {
            // Deviation: the result is ignored. Writing to a `Vec<u8>` never fails.
            let _ = write!(text, "{value}");
        })
    }

    /// Appends a `decimal128` value in base 10, without trailing zeros and never as `-0`.
    ///
    /// The text has `0` before the point if the integer part is zero. It has a `.` and the fraction
    /// only if the fraction is not zero. So the scale of the column does not change the text:
    /// `100.00` prints as `100`, and `-0.250` as `-0.25`.
    ///
    /// # Panics
    ///
    /// Panics if the text is longer than `u32::MAX` bytes, because the length prefix has 4 bytes.
    /// The text of a `decimal128` value has at most 38 digits, a sign, a point, and the 128 zeros of
    /// the most negative scale, -128. So it stays far below the limit. A `BigDecimal` from another
    /// source, such as `1E+5000000000`, can reach the panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::str::FromStr;
    ///
    /// use bigdecimal::BigDecimal;
    /// use sec::shared::row_digest::ByteForm;
    ///
    /// let value = BigDecimal::from_str("100.00")
    ///     .expect("Given a hardcoded decimal string, the parse should always succeed");
    ///
    /// // marker 01 | length 00000003 | text 100
    /// let expected_result: &[u8] = b"\x01\x00\x00\x00\x03100";
    ///
    /// let mut byte_form = ByteForm::new();
    /// let result = byte_form.decimal(&value).as_bytes();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn decimal(&mut self, value: &BigDecimal) -> &mut Self {
        let (unscaled, scale) = value.as_bigint_and_scale();
        let is_negative = unscaled.sign() == Sign::Minus;
        let digits = unscaled.magnitude().to_string();
        self.text(|text| write_decimal(text, is_negative, &digits, scale))
    }

    /// Appends a `date32` value as `YYYY-MM-DD`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chrono::NaiveDate;
    /// use sec::shared::row_digest::ByteForm;
    ///
    /// let period_end = NaiveDate::from_ymd_opt(2026, 11, 6)
    ///     .expect("Given a hardcoded valid date, the creation should always succeed");
    ///
    /// // marker 01 | length 0000000a | text 2026-11-06
    /// let expected_result: &[u8] = b"\x01\x00\x00\x00\x0a2026-11-06";
    ///
    /// let mut byte_form = ByteForm::new();
    /// let result = byte_form.date(period_end).as_bytes();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn date(&mut self, value: NaiveDate) -> &mut Self {
        self.text(|text| {
            // Deviation: the result is ignored. Writing to a `Vec<u8>` never fails.
            let _ = write!(
                text,
                "{:04}-{:02}-{:02}",
                value.year(),
                value.month(),
                value.day()
            );
        })
    }

    /// Appends a `timestamp[ms, +00:00]` value as `YYYY-MM-DDTHH:MM:SS.mmmZ`.
    ///
    /// The text drops the part of a second below one millisecond.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use chrono::{TimeZone, Utc};
    /// use sec::shared::row_digest::ByteForm;
    ///
    /// let accepted_at = Utc
    ///     .with_ymd_and_hms(2026, 11, 6, 13, 5, 0)
    ///     .single()
    ///     .expect("Given a hardcoded valid UTC time, the creation should always succeed");
    ///
    /// // marker 01 | length 00000018 | text 2026-11-06T13:05:00.000Z
    /// let expected_result: &[u8] = b"\x01\x00\x00\x00\x182026-11-06T13:05:00.000Z";
    ///
    /// let mut byte_form = ByteForm::new();
    /// let result = byte_form.timestamp(accepted_at).as_bytes();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn timestamp(&mut self, value: DateTime<Utc>) -> &mut Self {
        self.text(|text| {
            // Deviation: the result is ignored. Writing to a `Vec<u8>` never fails.
            let _ = write!(
                text,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                value.year(),
                value.month(),
                value.day(),
                value.hour(),
                value.minute(),
                value.second(),
                value.timestamp_subsec_millis()
            );
        })
    }

    /// Appends the head of a list value: the marker and the element count.
    ///
    /// The caller appends each element after the head, in the order of the list.
    ///
    /// # Panics
    ///
    /// Panics if the count is greater than `u32::MAX`, because the count has 4 bytes. No Arrow
    /// `list` value has such a count, because its offsets are 32-bit signed integers. A
    /// `large_list` column has 64-bit offsets and can reach the panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sec::shared::row_digest::ByteForm;
    ///
    /// let members = ["a", "b"];
    ///
    /// // list head: 01 00000002 | element 1: 01 00000001 a | element 2: 01 00000001 b
    /// let expected_result: &[u8] = b"\x01\x00\x00\x00\x02\x01\x00\x00\x00\x01a\x01\x00\x00\x00\x01b";
    ///
    /// let mut byte_form = ByteForm::new();
    /// byte_form.list(members.len());
    /// for member in members {
    ///     byte_form.utf8(member);
    /// }
    /// let result = byte_form.as_bytes();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn list(&mut self, element_count: usize) -> &mut Self {
        self.bytes.push(VALUE_MARKER);
        self.bytes.extend_from_slice(&length_prefix(element_count));
        self
    }

    /// Appends the head of a struct value: the marker and the field count.
    ///
    /// The caller appends each field after the head, in the order of the schema.
    ///
    /// # Panics
    ///
    /// Panics if the count is greater than `u32::MAX`, because the count has 4 bytes. The count is
    /// the number of fields in the schema of the struct, so it is never greater than `u32::MAX`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sec::shared::row_digest::ByteForm;
    ///
    /// let axis = "a";
    ///
    /// // struct head: 01 00000002 | field 1: 01 00000001 a | field 2: null 00
    /// let expected_result: &[u8] = b"\x01\x00\x00\x00\x02\x01\x00\x00\x00\x01a\x00";
    ///
    /// let mut byte_form = ByteForm::new();
    /// let result = byte_form.struct_(2).utf8(axis).null().as_bytes();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn struct_(&mut self, field_count: usize) -> &mut Self {
        self.bytes.push(VALUE_MARKER);
        self.bytes.extend_from_slice(&length_prefix(field_count));
        self
    }

    /// Appends a scalar: the marker, the 4-byte big-endian length, and the text that `print`
    /// writes.
    fn text(&mut self, print: impl FnOnce(&mut Vec<u8>)) -> &mut Self {
        self.bytes.push(VALUE_MARKER);
        let length_at = self.bytes.len();
        self.bytes.extend_from_slice(&[0; 4]);
        print(&mut self.bytes);
        let length = self.bytes.len() - length_at - 4;
        self.bytes[length_at..length_at + 4].copy_from_slice(&length_prefix(length));
        self
    }
}

/// Returns a length or a count as 4 big-endian bytes.
fn length_prefix(length: usize) -> [u8; 4] {
    u32::try_from(length)
        .expect("Given Arrow offsets of 32-bit signed integers, a length or a count should always fit in 32 bits")
        .to_be_bytes()
}

/// Writes the text of a decimal from the base-10 digits of its unscaled value and its scale.
///
/// Drops the trailing zeros first. A zero has no sign, so the text is never `-0`.
fn write_decimal(text: &mut Vec<u8>, is_negative: bool, digits: &str, scale: i64) {
    let significant = digits.trim_end_matches('0');
    if significant.is_empty() {
        text.push(b'0');
        return;
    }
    let trailing_zeros = digits.len() - significant.len();
    let scale = scale - i64::try_from(trailing_zeros).unwrap_or(i64::MAX);
    if is_negative {
        text.push(b'-');
    }
    if scale <= 0 {
        text.extend_from_slice(significant.as_bytes());
        text.extend(std::iter::repeat_n(
            b'0',
            usize::try_from(scale.unsigned_abs()).unwrap_or(usize::MAX),
        ));
        return;
    }
    let fraction_length = usize::try_from(scale).unwrap_or(usize::MAX);
    if fraction_length < significant.len() {
        let (integer_part, fraction) = significant.split_at(significant.len() - fraction_length);
        text.extend_from_slice(integer_part.as_bytes());
        text.push(b'.');
        text.extend_from_slice(fraction.as_bytes());
    } else {
        text.extend_from_slice(b"0.");
        text.extend(std::iter::repeat_n(
            b'0',
            fraction_length - significant.len(),
        ));
        text.extend_from_slice(significant.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;
    use std::str::FromStr;

    use chrono::TimeZone;
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;

    /// Returns the bytes of one value that `append` writes to an empty [`ByteForm`].
    fn bytes_of(append: impl FnOnce(&mut ByteForm)) -> Vec<u8> {
        let mut byte_form = ByteForm::new();
        append(&mut byte_form);
        byte_form.into_bytes()
    }

    /// Returns the expected bytes of a scalar with the given text.
    fn scalar(text: &str) -> Vec<u8> {
        let mut bytes = vec![VALUE_MARKER];
        bytes.extend_from_slice(
            &u32::try_from(text.len())
                .expect("Given short test strings, the length should always fit in 32 bits")
                .to_be_bytes(),
        );
        bytes.extend_from_slice(text.as_bytes());
        bytes
    }

    /// Returns the text of a decimal, parsed from a decimal string.
    fn decimal_text(decimal_string: &str) -> String {
        let decimal = BigDecimal::from_str(decimal_string)
            .expect("Given a hardcoded decimal string, the parse should always succeed");
        let bytes = bytes_of(|byte_form| {
            byte_form.decimal(&decimal);
        });
        String::from_utf8(bytes[5..].to_vec())
            .expect("Given a decimal printed as ASCII digits, the text should always be UTF-8")
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_byte_form() {
        implements_auto_traits::<ByteForm>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Default + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_byte_form() {
        implements_value_traits::<ByteForm>();
    }

    #[test]
    fn should_hold_no_bytes_when_no_value_is_appended() {
        let byte_form = ByteForm::new();

        let expected_result: &[u8] = &[];

        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_null_marker_only_when_the_value_is_null() {
        let mut byte_form = ByteForm::new();

        let expected_result: &[u8] = &[0x00];

        let result = byte_form.null().as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_marker_length_and_text_when_the_value_is_utf8() {
        let value = "us-gaap";

        let expected_result = vec![0x01, 0, 0, 0, 7, b'u', b's', b'-', b'g', b'a', b'a', b'p'];

        let result = bytes_of(|byte_form| {
            byte_form.utf8(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_a_zero_length_when_the_utf8_value_is_empty() {
        let value = "";

        let expected_result = vec![0x01, 0, 0, 0, 0];

        let result = bytes_of(|byte_form| {
            byte_form.utf8(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_byte_length_when_the_utf8_value_has_multibyte_characters() {
        let value = "€";

        let expected_result = vec![0x01, 0, 0, 0, 3, 0xE2, 0x82, 0xAC];

        let result = bytes_of(|byte_form| {
            byte_form.utf8(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_a_big_endian_length_when_the_utf8_value_is_longer_than_255_bytes() {
        let value = "a".repeat(258);

        let expected_result = vec![0x01, 0, 0, 1, 2];

        let result = bytes_of(|byte_form| {
            byte_form.utf8(&value);
        })[..5]
            .to_vec();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_true_when_the_bool_value_is_true() {
        let value = true;

        let expected_result = scalar("true");

        let result = bytes_of(|byte_form| {
            byte_form.bool(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_false_when_the_bool_value_is_false() {
        let value = false;

        let expected_result = scalar("false");

        let result = bytes_of(|byte_form| {
            byte_form.bool(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_base_10_when_the_integer_is_positive() {
        let value: u16 = 4007;

        let expected_result = scalar("4007");

        let result = bytes_of(|byte_form| {
            byte_form.integer(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_a_minus_sign_when_the_integer_is_negative() {
        let value: i64 = -32768;

        let expected_result = scalar("-32768");

        let result = bytes_of(|byte_form| {
            byte_form.integer(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_0_when_the_integer_is_zero() {
        let value: i32 = 0;

        let expected_result = scalar("0");

        let result = bytes_of(|byte_form| {
            byte_form.integer(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_same_bytes_when_an_integer_column_widens() {
        let wide_value: u16 = 4;

        let expected_result = scalar("4");

        let result = bytes_of(|byte_form| {
            byte_form.integer(wide_value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_largest_value_when_the_integer_is_u64_max() {
        let value = u64::MAX;

        let expected_result = scalar("18446744073709551615");

        let result = bytes_of(|byte_form| {
            byte_form.integer(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_marker_and_length_when_the_value_is_a_decimal() {
        let value = BigDecimal::from_str("-0.25")
            .expect("Given a hardcoded decimal string, the parse should always succeed");

        let expected_result = scalar("-0.25");

        let result = bytes_of(|byte_form| {
            byte_form.decimal(&value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_trailing_zeros_when_the_decimal_fraction_ends_in_zeros() {
        let decimal_string = "1234.5000";

        let expected_result = "1234.5";

        let result = decimal_text(decimal_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_the_point_when_the_decimal_fraction_is_zero() {
        let decimal_string = "100.00";

        let expected_result = "100";

        let result = decimal_text(decimal_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_zeros_of_the_integer_part_when_the_decimal_has_a_negative_scale() {
        let decimal_string = "1E+3";

        let expected_result = "1000";

        let result = decimal_text(decimal_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_0_before_the_point_when_the_decimal_integer_part_is_zero() {
        let decimal_string = "0.25";

        let expected_result = "0.25";

        let result = decimal_text(decimal_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_leading_fraction_zeros_when_the_decimal_is_smaller_than_a_tenth() {
        let decimal_string = "0.00500";

        let expected_result = "0.005";

        let result = decimal_text(decimal_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_a_minus_sign_when_the_decimal_is_negative() {
        let decimal_string = "-1234.50";

        let expected_result = "-1234.5";

        let result = decimal_text(decimal_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_0_when_the_decimal_is_negative_zero() {
        let decimal_string = "-0.00";

        let expected_result = "0";

        let result = decimal_text(decimal_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_0_when_the_decimal_is_zero_with_a_negative_scale() {
        let decimal_string = "0E+5";

        let expected_result = "0";

        let result = decimal_text(decimal_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_same_bytes_when_the_decimal_scale_changes() {
        let wide_scale = BigDecimal::from_str("12.5000000000")
            .expect("Given a hardcoded decimal string, the parse should always succeed");

        let expected_result = scalar("12.5");

        let result = bytes_of(|byte_form| {
            byte_form.decimal(&wide_scale);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_all_38_digits_when_the_decimal_has_the_full_decimal128_precision() {
        let decimal_string = "-1234567890123456789.0123456789012345678";

        let expected_result = "-1234567890123456789.0123456789012345678";

        let result = decimal_text(decimal_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_all_digits_when_the_decimal_has_more_than_38_digits() {
        let decimal_string = "-123456789012345678901234567890123456789012.50";

        let expected_result = "-123456789012345678901234567890123456789012.5";

        let result = decimal_text(decimal_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_yyyy_mm_dd_when_the_value_is_a_date() {
        let value = NaiveDate::from_ymd_opt(2026, 11, 6)
            .expect("Given a hardcoded valid date, the creation should always succeed");

        let expected_result = scalar("2026-11-06");

        let result = bytes_of(|byte_form| {
            byte_form.date(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_pad_the_year_to_four_digits_when_the_date_is_before_year_1000() {
        let value = NaiveDate::from_ymd_opt(999, 1, 2)
            .expect("Given a hardcoded valid date, the creation should always succeed");

        let expected_result = scalar("0999-01-02");

        let result = bytes_of(|byte_form| {
            byte_form.date(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_milliseconds_and_z_when_the_value_is_a_timestamp() {
        let value = Utc
            .with_ymd_and_hms(2026, 11, 6, 13, 5, 0)
            .single()
            .expect("Given a hardcoded valid UTC time, the creation should always succeed");

        let expected_result = scalar("2026-11-06T13:05:00.000Z");

        let result = bytes_of(|byte_form| {
            byte_form.timestamp(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_the_part_below_one_millisecond_when_the_timestamp_has_microseconds() {
        let value = Utc
            .timestamp_opt(1_762_434_300, 123_456_789)
            .single()
            .expect("Given a hardcoded valid Unix time, the creation should always succeed");

        let expected_result = scalar("2025-11-06T13:05:00.123Z");

        let result = bytes_of(|byte_form| {
            byte_form.timestamp(value);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_marker_and_element_count_before_the_elements_when_the_value_is_a_list() {
        let elements = ["a", "b"];

        let expected_result = vec![
            0x01, 0, 0, 0, 2, 0x01, 0, 0, 0, 1, b'a', 0x01, 0, 0, 0, 1, b'b',
        ];

        let result = bytes_of(|byte_form| {
            byte_form.list(elements.len());
            for element in elements {
                byte_form.utf8(element);
            }
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_marker_and_a_zero_count_when_the_list_is_empty() {
        let element_count = 0;

        let expected_result = vec![0x01, 0, 0, 0, 0];

        let result = bytes_of(|byte_form| {
            byte_form.list(element_count);
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_marker_and_field_count_before_the_fields_when_the_value_is_a_struct() {
        let axis = "srt:ProductOrServiceAxis";
        let member: Option<&str> = None;

        let expected_result = vec![
            0x01, 0, 0, 0, 2, 0x01, 0, 0, 0, 24, b's', b'r', b't', b':', b'P', b'r', b'o', b'd',
            b'u', b'c', b't', b'O', b'r', b'S', b'e', b'r', b'v', b'i', b'c', b'e', b'A', b'x',
            b'i', b's', 0x00,
        ];

        let result = bytes_of(|byte_form| {
            byte_form.struct_(2).utf8(axis);
            match member {
                Some(member) => byte_form.utf8(member),
                None => byte_form.null(),
            };
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_different_bytes_when_the_same_text_splits_across_two_values_differently() {
        let first_split = ["ab", "c"];
        let second_split = ["a", "bc"];

        let expected_result = bytes_of(|byte_form| {
            byte_form.utf8(first_split[0]).utf8(first_split[1]);
        });

        let result = bytes_of(|byte_form| {
            byte_form.utf8(second_split[0]).utf8(second_split[1]);
        });

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_give_different_bytes_when_a_value_is_null_or_an_empty_string() {
        let empty = "";

        let expected_result = bytes_of(|byte_form| {
            byte_form.null();
        });

        let result = bytes_of(|byte_form| {
            byte_form.utf8(empty);
        });

        assert_ne!(result, expected_result);
    }
}
