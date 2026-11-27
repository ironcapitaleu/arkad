//! # New York Clock
//!
//! Provides [`today_in_new_york`] and [`previous_day_in_new_york`], which read a day in New York
//! (`America/New_York`) from a time of the clock.
//!
//! The SEC's days are Eastern. It publishes the daily index of a day in the evening of that day or
//! later, so the index of today can be missing.
//!
//! The zone data of New York is compiled into the binary, so no binary reads the zone files of the
//! image.

use jiff::civil::Date;
use jiff::tz::{self, TimeZone};
use jiff::{Timestamp, ToSpan};

/// The time zone of New York, compiled in from the tz database that `jiff` bundles.
static NEW_YORK: TimeZone = tz::get!("America/New_York");

/// Returns today in New York at the time `now`.
pub fn today_in_new_york(now: Timestamp) -> Date {
    NEW_YORK.to_datetime(now).date()
}

/// Returns the day before today in New York at the time `now`.
///
/// # Panics
///
/// Panics if today in New York is the first day that [`Date`] holds. A reading of the clock never
/// gives that day.
pub fn previous_day_in_new_york(now: Timestamp) -> Date {
    today_in_new_york(now)
        .checked_sub(1.day())
        .expect("Given a date of the current era, the day before it always exists")
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_the_previous_day_in_new_york_when_utc_is_already_the_next_day() {
        // 23:00 on 31 October in New York (UTC-4). In UTC it is already 1 November.
        let now: Timestamp = "2026-11-01T03:00:00Z"
            .parse()
            .expect("Given a hardcoded RFC 3339 time, parsing should always succeed");

        let expected_result = date(2026, 10, 30);

        let result = previous_day_in_new_york(now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_today_in_new_york_when_utc_is_already_the_next_day_on_standard_time() {
        // 21:30 on 15 January in New York (UTC-5). In UTC it is already 16 January.
        let now: Timestamp = "2026-01-16T02:30:00Z"
            .parse()
            .expect("Given a hardcoded RFC 3339 time, parsing should always succeed");

        let expected_result = date(2026, 1, 15);

        let result = today_in_new_york(now);

        assert_eq!(result, expected_result);
    }
}
