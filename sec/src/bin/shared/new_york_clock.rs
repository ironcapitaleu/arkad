//! # New York Clock
//!
//! Provides [`previous_day_in_new_york`], which reads the day before today in New York
//! (`America/New_York`) from a time of the clock.
//!
//! The SEC's days are Eastern. It publishes the daily index of a day in the evening of that day or
//! later, so the index of today can be missing.

use chrono::{DateTime, Days, NaiveDate, Utc};
use chrono_tz::America::New_York;

/// Returns the day before today in New York at the time `now`.
pub fn previous_day_in_new_york(now: DateTime<Utc>) -> NaiveDate {
    now.with_timezone(&New_York)
        .date_naive()
        .checked_sub_days(Days::new(1))
        .expect("Given a date of the current era, the day before it always exists")
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_the_previous_day_in_new_york_when_utc_is_already_the_next_day() {
        // 23:00 on 31 October in New York (UTC-4). In UTC it is already 1 November.
        let now = DateTime::parse_from_rfc3339("2026-11-01T03:00:00Z")
            .expect("Given a hardcoded RFC 3339 time, parsing should always succeed")
            .with_timezone(&Utc);

        let expected_result = NaiveDate::from_ymd_opt(2026, 10, 30)
            .expect("Given a hardcoded calendar date, building it should always succeed");

        let result = previous_day_in_new_york(now);

        assert_eq!(result, expected_result);
    }
}
