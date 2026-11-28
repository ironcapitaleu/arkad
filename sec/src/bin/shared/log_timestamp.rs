//! # Log Timestamp
//!
//! Writes the `timestamp` field of a log line. `json_log.rs` and the `extraction` binary use it,
//! so every binary writes the same form.

use jiff::Timestamp;

/// Writes `time` in RFC 3339 in UTC, with the fraction truncated to six digits, as
/// `2024-10-12T14:30:00.000000Z`.
pub fn log_timestamp(time: Timestamp) -> String {
    format!("{time:.6}")
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;
    use jiff::tz::TimeZone;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_write_six_fraction_digits_when_the_time_has_nanoseconds() {
        let time = TimeZone::UTC
            .to_timestamp(date(2024, 10, 12).at(14, 30, 0, 123_456_789))
            .expect("Given a valid hardcoded UTC time, building it should always succeed");

        let expected_result = "2024-10-12T14:30:00.123456Z";

        let result = log_timestamp(time);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_six_zero_fraction_digits_when_the_time_is_a_whole_second() {
        let time = TimeZone::UTC
            .to_timestamp(date(2024, 10, 12).at(14, 30, 0, 0))
            .expect("Given a valid hardcoded UTC time, building it should always succeed");

        let expected_result = "2024-10-12T14:30:00.000000Z";

        let result = log_timestamp(time);

        assert_eq!(result, expected_result);
    }
}
