//! # Stored Timestamp
//!
//! Provides the serde functions that write a [`Timestamp`] in the stored form, and read it back.
//!
//! The stored form is RFC 3339 in UTC with the suffix `Z`. The fraction of a second has no digits
//! for a whole second, else 3, 6, or 9 digits: the fewest of these that hold the nanoseconds. For
//! example, `2026-09-22T20:05:30Z`, `2026-09-22T20:05:30.500Z`, and
//! `2026-09-22T20:05:30.123400Z`.
//! [ADR 0047](https://github.com/ironcapitaleu/arkad/blob/main/docs/adr/0047-jiff-for-dates-and-time-zones.md),
//! decision 4 and amendment 1, fix this form for every stored timestamp.
//!
//! A field uses the functions with `#[serde(with = "crate::shared::stored_timestamp")]`.

use std::fmt;

use jiff::Timestamp;
use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};

/// Writes the timestamp in the stored form.
///
/// # Errors
///
/// Returns the error of the serializer if it cannot write a string.
pub fn serialize<S: Serializer>(timestamp: &Timestamp, serializer: S) -> Result<S::Ok, S::Error> {
    let digits = fraction_digits(*timestamp);
    serializer.collect_str(&format_args!("{timestamp:.digits$}"))
}

/// Reads a timestamp from an RFC 3339 string with any offset and any number of fraction digits.
///
/// # Errors
///
/// Returns a deserializer error if the value is not a string, or if the string is not an RFC 3339
/// timestamp in the range of [`Timestamp`].
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Timestamp, D::Error> {
    deserializer.deserialize_str(TimestampVisitor)
}

/// Returns the number of fraction digits of the stored form: 0, 3, 6, or 9.
#[must_use]
pub fn fraction_digits(timestamp: Timestamp) -> usize {
    let nanoseconds = timestamp.subsec_nanosecond();
    if nanoseconds == 0 {
        0
    } else if nanoseconds % 1_000_000 == 0 {
        3
    } else if nanoseconds % 1_000 == 0 {
        6
    } else {
        9
    }
}

/// Parses a [`Timestamp`] from a string.
struct TimestampVisitor;

impl Visitor<'_> for TimestampVisitor {
    type Value = Timestamp;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an RFC 3339 timestamp")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Timestamp, E> {
        value.parse().map_err(E::custom)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Stored(#[serde(with = "super")] Timestamp);

    fn timestamp(text: &str) -> Timestamp {
        text.parse()
            .expect("Given a hardcoded RFC 3339 timestamp, parsing it should always succeed")
    }

    fn written(text: &str) -> String {
        serde_json::to_string(&Stored(timestamp(text)))
            .expect("Given a timestamp, serialization should always succeed")
    }

    #[test]
    fn should_write_no_fraction_when_the_timestamp_is_a_whole_second() {
        let text = "2026-09-22T20:05:30.000Z";

        let expected_result = r#""2026-09-22T20:05:30Z""#;

        let result = written(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_pad_the_fraction_to_milliseconds_when_the_timestamp_has_tenths_of_a_second() {
        let text = "2026-09-22T20:05:30.5Z";

        let expected_result = r#""2026-09-22T20:05:30.500Z""#;

        let result = written(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_pad_the_fraction_to_microseconds_when_the_timestamp_has_tenths_of_a_millisecond() {
        let text = "2026-09-22T20:05:30.1234Z";

        let expected_result = r#""2026-09-22T20:05:30.123400Z""#;

        let result = written(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_pad_the_fraction_to_nanoseconds_when_the_timestamp_has_tenths_of_a_microsecond() {
        let text = "2026-09-22T20:05:30.1234567Z";

        let expected_result = r#""2026-09-22T20:05:30.123456700Z""#;

        let result = written(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_leading_zeros_when_the_timestamp_has_one_nanosecond() {
        let text = "2026-09-22T20:05:30.000000001Z";

        let expected_result = r#""2026-09-22T20:05:30.000000001Z""#;

        let result = written(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_leading_zeros_when_the_timestamp_has_half_a_millisecond() {
        let text = "2026-09-22T20:05:30.0005Z";

        let expected_result = r#""2026-09-22T20:05:30.000500Z""#;

        let result = written(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_pad_the_fraction_to_milliseconds_when_the_timestamp_is_before_the_unix_epoch() {
        let text = "1969-12-31T23:59:59.5Z";

        let expected_result = r#""1969-12-31T23:59:59.500Z""#;

        let result = written(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_the_instant_in_utc_when_the_stored_timestamp_has_an_offset() {
        let stored = r#""2026-09-22T22:05:30+02:00""#;

        let expected_result = Stored(timestamp("2026-09-22T20:05:30Z"));

        let result: Stored = serde_json::from_str(stored)
            .expect("Given an RFC 3339 timestamp, deserialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_read_when_the_stored_timestamp_has_no_offset() {
        let stored = r#""2026-09-22T20:05:30""#;

        let expected_result = Some(
            "failed to find offset component, which is required for parsing a timestamp at line 1 column 21"
                .to_owned(),
        );

        let result = serde_json::from_str::<Stored>(stored)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }
}
