//! # Dates
//!
//! Provides [`run_dates`], which turns the values of [`FROM_ENV_VAR`] and [`TO_ENV_VAR`] into the
//! dates of one run.
//!
//! Every date is a day in New York (`America/New_York`). The SEC's days are Eastern, and it
//! publishes the daily index of a day only after that day ends there. At 02:00 UTC on 2 November it
//! is still 1 November in New York, so [`YESTERDAY`] is 31 October.

use std::ffi::OsString;

use chrono::{DateTime, Days, NaiveDate, Utc};
use chrono_tz::America::New_York;
use thiserror::Error;

use crate::constants::{FROM_ENV_VAR, TO_ENV_VAR, YESTERDAY};

/// The error of values of [`FROM_ENV_VAR`] and [`TO_ENV_VAR`] that give no dates to run.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum InvalidRunDates {
    /// [`FROM_ENV_VAR`] is unset or empty.
    #[error(
        "[MissingStartDate] {FROM_ENV_VAR} is unset, set {FROM_ENV_VAR} and optionally {TO_ENV_VAR} to a date (YYYY-MM-DD) or '{YESTERDAY}'"
    )]
    MissingStartDate,
    /// A value is neither a date in the form `YYYY-MM-DD` nor [`YESTERDAY`].
    #[error(
        "[InvalidDate] {variable} value '{value}' is not a date (YYYY-MM-DD) or '{YESTERDAY}', set {FROM_ENV_VAR} and {TO_ENV_VAR} in that form"
    )]
    InvalidDate {
        /// The variable that holds the value.
        variable: &'static str,
        /// The value as the environment gave it, with invalid Unicode replaced.
        value: String,
    },
    /// The last date is today or later in New York, so its daily index does not exist yet.
    #[error(
        "[InvalidDateRange] {TO_ENV_VAR} date '{to}' is not before today '{today}' in New York, the SEC publishes a daily index only after its day ends"
    )]
    LastDateNotPast {
        /// The last date of the run.
        to: NaiveDate,
        /// Today in New York.
        today: NaiveDate,
    },
    /// The first date comes after the last date.
    #[error("[InvalidDateRange] {FROM_ENV_VAR} date '{from}' comes after {TO_ENV_VAR} date '{to}'")]
    FirstDateAfterLastDate {
        /// The first date of the run.
        from: NaiveDate,
        /// The last date of the run.
        to: NaiveDate,
    },
}

/// Returns every date from `from` through `to`, oldest first, for the values of [`FROM_ENV_VAR`]
/// and [`TO_ENV_VAR`] at the time `now`.
///
/// Each value is a date in the form `YYYY-MM-DD` or [`YESTERDAY`], the day before today in New
/// York. An unset or empty `to` means [`YESTERDAY`].
///
/// # Errors
///
/// Returns an [`InvalidRunDates`]:
/// - [`InvalidRunDates::MissingStartDate`] if `from` is unset or empty.
/// - [`InvalidRunDates::InvalidDate`] if a value is neither a date nor [`YESTERDAY`].
/// - [`InvalidRunDates::LastDateNotPast`] if the last date is today or later in New York.
/// - [`InvalidRunDates::FirstDateAfterLastDate`] if the first date comes after the last date.
pub fn run_dates(
    from: Option<OsString>,
    to: Option<OsString>,
    now: DateTime<Utc>,
) -> Result<Vec<NaiveDate>, InvalidRunDates> {
    let today = now.with_timezone(&New_York).date_naive();
    let yesterday = today
        .checked_sub_days(Days::new(1))
        .expect("Given a date of the current era, the day before it always exists");

    let from = non_empty(from).ok_or(InvalidRunDates::MissingStartDate)?;
    let from = parse_date(FROM_ENV_VAR, &from, yesterday)?;
    let to = match non_empty(to) {
        Some(to) => parse_date(TO_ENV_VAR, &to, yesterday)?,
        None => yesterday,
    };

    if to >= today {
        return Err(InvalidRunDates::LastDateNotPast { to, today });
    }
    if from > to {
        return Err(InvalidRunDates::FirstDateAfterLastDate { from, to });
    }
    Ok(from.iter_days().take_while(|date| *date <= to).collect())
}

/// Returns the value as a string, or `None` if it is unset or empty.
fn non_empty(value: Option<OsString>) -> Option<String> {
    value
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string_lossy().into_owned())
}

/// Parses `value` of `variable` as a date in the form `YYYY-MM-DD` or as [`YESTERDAY`].
fn parse_date(
    variable: &'static str,
    value: &str,
    yesterday: NaiveDate,
) -> Result<NaiveDate, InvalidRunDates> {
    if value == YESTERDAY {
        return Ok(yesterday);
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| InvalidRunDates::InvalidDate {
        variable,
        value: value.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    /// 12:00 UTC on 2026-11-01, which is 07:00 on 2026-11-01 in New York.
    fn noon_utc() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-01T12:00:00Z")
            .expect("Given a hardcoded RFC 3339 time, parsing should always succeed")
            .with_timezone(&Utc)
    }

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .expect("Given a hardcoded valid date, parsing should always succeed")
    }

    #[test]
    fn should_return_yesterday_in_new_york_when_from_is_yesterday_and_to_is_unset() {
        let from = Some(OsString::from("yesterday"));

        let expected_result = Ok(vec![date("2026-10-31")]);

        let result = run_dates(from, None, noon_utc());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_day_before_in_new_york_when_it_is_already_the_next_day_in_utc() {
        let from = Some(OsString::from("yesterday"));
        let two_in_the_morning_utc = DateTime::parse_from_rfc3339("2026-11-02T02:00:00Z")
            .expect("Given a hardcoded RFC 3339 time, parsing should always succeed")
            .with_timezone(&Utc);

        let expected_result = Ok(vec![date("2026-10-31")]);

        let result = run_dates(from, None, two_in_the_morning_utc);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_every_date_up_to_yesterday_when_only_from_is_set() {
        let from = Some(OsString::from("2026-10-28"));

        let expected_result = Ok(vec![
            date("2026-10-28"),
            date("2026-10-29"),
            date("2026-10-30"),
            date("2026-10-31"),
        ]);

        let result = run_dates(from, None, noon_utc());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_one_date_when_from_and_to_are_the_same_date() {
        let from = Some(OsString::from("2026-10-15"));
        let to = Some(OsString::from("2026-10-15"));

        let expected_result = Ok(vec![date("2026-10-15")]);

        let result = run_dates(from, to, noon_utc());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_every_date_of_the_range_when_from_and_to_are_set() {
        let from = Some(OsString::from("2026-10-13"));
        let to = Some(OsString::from("2026-10-15"));

        let expected_result = Ok(vec![
            date("2026-10-13"),
            date("2026-10-14"),
            date("2026-10-15"),
        ]);

        let result = run_dates(from, to, noon_utc());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_yesterday_when_to_is_yesterday() {
        let from = Some(OsString::from("2026-10-31"));
        let to = Some(OsString::from("yesterday"));

        let expected_result = Ok(vec![date("2026-10-31")]);

        let result = run_dates(from, to, noon_utc());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_start_date_when_from_is_unset() {
        let to = Some(OsString::from("2026-10-15"));

        let expected_result = Err(InvalidRunDates::MissingStartDate);

        let result = run_dates(None, to, noon_utc());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_start_date_when_from_is_empty() {
        let from = Some(OsString::from(""));

        let expected_result = Err(InvalidRunDates::MissingStartDate);

        let result = run_dates(from, None, noon_utc());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_when_from_does_not_parse() {
        let from = Some(OsString::from("2026-13-01"));

        let expected_result = Err(InvalidRunDates::InvalidDate {
            variable: "ARKAD_FROM",
            value: "2026-13-01".to_string(),
        });

        let result = run_dates(from, None, noon_utc());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_when_to_does_not_parse() {
        let from = Some(OsString::from("2026-10-15"));
        let to = Some(OsString::from("today"));

        let expected_result = Err(InvalidRunDates::InvalidDate {
            variable: "ARKAD_TO",
            value: "today".to_string(),
        });

        let result = run_dates(from, to, noon_utc());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_last_date_not_past_when_to_is_today_in_new_york() {
        let from = Some(OsString::from("2026-10-31"));
        let to = Some(OsString::from("2026-11-01"));

        let expected_result = Err(InvalidRunDates::LastDateNotPast {
            to: date("2026-11-01"),
            today: date("2026-11-01"),
        });

        let result = run_dates(from, to, noon_utc());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_first_date_after_last_date_when_from_is_after_to() {
        let from = Some(OsString::from("2026-10-20"));
        let to = Some(OsString::from("2026-10-19"));

        let expected_result = Err(InvalidRunDates::FirstDateAfterLastDate {
            from: date("2026-10-20"),
            to: date("2026-10-19"),
        });

        let result = run_dates(from, to, noon_utc());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_first_date_after_last_date_when_from_is_today_and_to_is_unset() {
        let from = Some(OsString::from("2026-11-01"));

        let expected_result = Err(InvalidRunDates::FirstDateAfterLastDate {
            from: date("2026-11-01"),
            to: date("2026-10-31"),
        });

        let result = run_dates(from, None, noon_utc());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_both_variables_when_the_start_date_is_missing() {
        let error = InvalidRunDates::MissingStartDate;

        let expected_result = "[MissingStartDate] ARKAD_FROM is unset, set ARKAD_FROM and optionally ARKAD_TO to a date (YYYY-MM-DD) or 'yesterday'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_variable_and_the_value_when_a_date_does_not_parse() {
        let error = InvalidRunDates::InvalidDate {
            variable: "ARKAD_TO",
            value: "today".to_string(),
        };

        let expected_result = "[InvalidDate] ARKAD_TO value 'today' is not a date (YYYY-MM-DD) or 'yesterday', set ARKAD_FROM and ARKAD_TO in that form";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
