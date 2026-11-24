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

use chrono::{DateTime, Days, NaiveDate, Utc};
use jiff::Timestamp;
use jiff::tz::{self, TimeZone};

/// The time zone of New York, compiled in from the tz database that `jiff` bundles.
static NEW_YORK: TimeZone = tz::get!("America/New_York");

/// Returns today in New York at the time `now`.
pub fn today_in_new_york(now: DateTime<Utc>) -> NaiveDate {
    // The fraction of the second cannot change the date, so the whole seconds are enough.
    let timestamp = Timestamp::from_second(now.timestamp()).expect(
        "Given a time of the current era, it is always inside the range of a jiff timestamp",
    );
    let date = NEW_YORK.to_datetime(timestamp).date();
    NaiveDate::from_ymd_opt(
        i32::from(date.year()),
        u32::from(date.month().unsigned_abs()),
        u32::from(date.day().unsigned_abs()),
    )
    .expect("Given a valid jiff date, the same calendar date always exists in chrono")
}

/// Returns the day before today in New York at the time `now`.
pub fn previous_day_in_new_york(now: DateTime<Utc>) -> NaiveDate {
    today_in_new_york(now)
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

    #[test]
    fn should_return_today_in_new_york_when_utc_is_already_the_next_day_on_standard_time() {
        // 21:30 on 15 January in New York (UTC-5). In UTC it is already 16 January.
        let now = DateTime::parse_from_rfc3339("2026-01-16T02:30:00Z")
            .expect("Given a hardcoded RFC 3339 time, parsing should always succeed")
            .with_timezone(&Utc);

        let expected_result = NaiveDate::from_ymd_opt(2026, 1, 15)
            .expect("Given a hardcoded calendar date, building it should always succeed");

        let result = today_in_new_york(now);

        assert_eq!(result, expected_result);
    }
}
