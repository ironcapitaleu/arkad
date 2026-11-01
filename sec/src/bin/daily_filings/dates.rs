//! # Dates
//!
//! Provides [`run_dates`], which turns the values of [`FROM_ENV_VAR`] and [`TO_ENV_VAR`] into the
//! dates of one run.
//!
//! Every date is a day in New York (`America/New_York`). The SEC's days are Eastern, and it
//! publishes the daily index of a day only after that day ends there. At 02:00 UTC on 2 November it
//! is still 1 November in New York, so [`YESTERDAY`] is 31 October.
//!
//! `yesterday-N` names the day `N` days before yesterday. The scheduled run sets
//! `ARKAD_FROM=yesterday-3`, so each run reads the last 4 days.

use std::ffi::OsString;

use chrono::{DateTime, Days, NaiveDate, Utc};
use chrono_tz::America::New_York;
use thiserror::Error;

use crate::constants::{FROM_ENV_VAR, TO_ENV_VAR, YESTERDAY};

/// The error when [`FROM_ENV_VAR`] and [`TO_ENV_VAR`] give no valid dates to run.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum InvalidRunDates {
    /// [`FROM_ENV_VAR`] is unset or empty.
    #[error(
        "[MissingStartDate] {FROM_ENV_VAR} is unset or empty, set {FROM_ENV_VAR} and optionally {TO_ENV_VAR} to a date (YYYY-MM-DD), '{YESTERDAY}' or '{YESTERDAY}-N'"
    )]
    MissingStartDate,
    /// A value is neither a date in the form `YYYY-MM-DD`, nor [`YESTERDAY`], nor `yesterday-N`.
    #[error(
        "[InvalidDate] {variable} value '{value}' is not a date (YYYY-MM-DD), '{YESTERDAY}' or '{YESTERDAY}-N', set {FROM_ENV_VAR} and {TO_ENV_VAR} in that form"
    )]
    InvalidDate {
        /// The variable that holds the value.
        variable: &'static str,
        /// The value as the environment gave it, with invalid Unicode replaced.
        value: String,
    },
    /// The last date is today or later in New York, so its daily index does not exist yet.
    #[error(
        "[InvalidLastDate] {TO_ENV_VAR} date '{to}' is not before today '{today}' in New York, the SEC publishes a daily index only after its day ends"
    )]
    InvalidLastDate {
        /// The last date of the run.
        to: NaiveDate,
        /// Today in New York.
        today: NaiveDate,
    },
    /// The first date comes after the last date.
    #[error("[InvalidDateOrder] {FROM_ENV_VAR} date '{from}' comes after the last date '{to}'")]
    InvalidDateOrder {
        /// The first date of the run.
        from: NaiveDate,
        /// The last date of the run.
        to: NaiveDate,
    },
}

/// Returns every date from `from` through `to`, oldest first, for the values of [`FROM_ENV_VAR`]
/// and [`TO_ENV_VAR`] at the time `now`.
///
/// Each value is a date in the form `YYYY-MM-DD`, [`YESTERDAY`] (the day before today in New
/// York), or `yesterday-N` (`N` days before yesterday, `N` at least 1). An unset or empty `to`
/// means [`YESTERDAY`].
///
/// # Errors
///
/// Returns an [`InvalidRunDates`]:
/// - [`InvalidRunDates::MissingStartDate`] if `from` is unset or empty.
/// - [`InvalidRunDates::InvalidDate`] if a value is none of the three forms.
/// - [`InvalidRunDates::InvalidLastDate`] if the last date is today or later in New York.
/// - [`InvalidRunDates::InvalidDateOrder`] if the first date comes after the last date.
pub fn run_dates(
    from: Option<OsString>,
    to: Option<OsString>,
    now: DateTime<Utc>,
) -> Result<Vec<NaiveDate>, InvalidRunDates> {
    let today = now.with_timezone(&New_York).date_naive();
    let yesterday = new_york_yesterday(now);

    let from = non_empty(from).ok_or(InvalidRunDates::MissingStartDate)?;
    let from = parse_date(FROM_ENV_VAR, &from, yesterday)?;
    let to = match non_empty(to) {
        Some(to) => parse_date(TO_ENV_VAR, &to, yesterday)?,
        None => yesterday,
    };

    if to >= today {
        return Err(InvalidRunDates::InvalidLastDate { to, today });
    }
    if from > to {
        return Err(InvalidRunDates::InvalidDateOrder { from, to });
    }
    Ok(from.iter_days().take_while(|date| *date <= to).collect())
}

/// Returns the day before today in New York at the time `now`.
pub fn new_york_yesterday(now: DateTime<Utc>) -> NaiveDate {
    now.with_timezone(&New_York)
        .date_naive()
        .checked_sub_days(Days::new(1))
        .expect("Given a date of the current era, the day before it always exists")
}

/// Returns the value as a string, or `None` if it is unset or empty.
fn non_empty(value: Option<OsString>) -> Option<String> {
    value
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string_lossy().into_owned())
}

/// Parses `value` of `variable` as a date in the form `YYYY-MM-DD`, as [`YESTERDAY`], or as
/// `yesterday-N`.
fn parse_date(
    variable: &'static str,
    value: &str,
    yesterday: NaiveDate,
) -> Result<NaiveDate, InvalidRunDates> {
    let date = if value == YESTERDAY {
        Some(yesterday)
    } else if let Some(days) = value
        .strip_prefix(YESTERDAY)
        .and_then(|rest| rest.strip_prefix('-'))
    {
        days_before(yesterday, days)
    } else {
        calendar_date(value)
    };
    date.ok_or_else(|| InvalidRunDates::InvalidDate {
        variable,
        value: value.to_string(),
    })
}

/// Returns the date `days` days before `yesterday`, or `None` if `days` is not a whole number of
/// at least 1 written without a sign or a leading zero.
fn days_before(yesterday: NaiveDate, days: &str) -> Option<NaiveDate> {
    // `u64` also reads a `+` and leading zeros, so the number must print back to the same text.
    days.parse::<u64>()
        .ok()
        .filter(|count| *count >= 1 && count.to_string() == days)
        .and_then(|count| yesterday.checked_sub_days(Days::new(count)))
}

/// Returns the date written as `YYYY-MM-DD`, or `None` if `value` is not in that exact form.
fn calendar_date(value: &str) -> Option<NaiveDate> {
    // `%Y` also reads a sign, and `%m` and `%d` also read one digit, so the date must print back
    // to the same text.
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()
        .filter(|date| date.format("%Y-%m-%d").to_string() == value)
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
        let to = None;
        let now = noon_utc();

        let expected_result = Ok(vec![date("2026-10-31")]);

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_day_before_in_new_york_when_it_is_already_the_next_day_in_utc() {
        let from = Some(OsString::from("yesterday"));
        let now = DateTime::parse_from_rfc3339("2026-11-02T02:00:00Z")
            .expect("Given a hardcoded RFC 3339 time, parsing should always succeed")
            .with_timezone(&Utc);
        let to = None;

        let expected_result = Ok(vec![date("2026-10-31")]);

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_every_date_up_to_yesterday_when_only_from_is_set() {
        let from = Some(OsString::from("2026-10-28"));
        let to = None;
        let now = noon_utc();

        let expected_result = Ok(vec![
            date("2026-10-28"),
            date("2026-10-29"),
            date("2026-10-30"),
            date("2026-10-31"),
        ]);

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_one_date_when_from_and_to_are_the_same_date() {
        let from = Some(OsString::from("2026-10-15"));
        let to = Some(OsString::from("2026-10-15"));
        let now = noon_utc();

        let expected_result = Ok(vec![date("2026-10-15")]);

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_every_date_of_the_range_when_from_and_to_are_set() {
        let from = Some(OsString::from("2026-10-13"));
        let to = Some(OsString::from("2026-10-15"));
        let now = noon_utc();

        let expected_result = Ok(vec![
            date("2026-10-13"),
            date("2026-10-14"),
            date("2026-10-15"),
        ]);

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_yesterday_when_to_is_yesterday() {
        let from = Some(OsString::from("2026-10-31"));
        let to = Some(OsString::from("yesterday"));
        let now = noon_utc();

        let expected_result = Ok(vec![date("2026-10-31")]);

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_start_date_when_from_is_unset() {
        let to = Some(OsString::from("2026-10-15"));
        let from = None;
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::MissingStartDate);

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_start_date_when_from_is_empty() {
        let from = Some(OsString::from(""));
        let to = None;
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::MissingStartDate);

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_when_from_does_not_parse() {
        let from = Some(OsString::from("2026-13-01"));
        let to = None;
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::InvalidDate {
            variable: "ARKAD_FROM",
            value: "2026-13-01".to_string(),
        });

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_when_to_does_not_parse() {
        let from = Some(OsString::from("2026-10-15"));
        let to = Some(OsString::from("today"));
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::InvalidDate {
            variable: "ARKAD_TO",
            value: "today".to_string(),
        });

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_last_date_when_to_is_today_in_new_york() {
        let from = Some(OsString::from("2026-10-31"));
        let to = Some(OsString::from("2026-11-01"));
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::InvalidLastDate {
            to: date("2026-11-01"),
            today: date("2026-11-01"),
        });

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_order_when_from_is_after_to() {
        let from = Some(OsString::from("2026-10-20"));
        let to = Some(OsString::from("2026-10-19"));
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::InvalidDateOrder {
            from: date("2026-10-20"),
            to: date("2026-10-19"),
        });

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_order_when_from_is_today_and_to_is_unset() {
        let from = Some(OsString::from("2026-11-01"));
        let to = None;
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::InvalidDateOrder {
            from: date("2026-11-01"),
            to: date("2026-10-31"),
        });

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_both_variables_when_the_start_date_is_missing() {
        let error = InvalidRunDates::MissingStartDate;

        let expected_result = "[MissingStartDate] ARKAD_FROM is unset or empty, set ARKAD_FROM and optionally ARKAD_TO to a date (YYYY-MM-DD), 'yesterday' or 'yesterday-N'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_variable_and_the_value_when_a_date_does_not_parse() {
        let error = InvalidRunDates::InvalidDate {
            variable: "ARKAD_TO",
            value: "today".to_string(),
        };

        let expected_result = "[InvalidDate] ARKAD_TO value 'today' is not a date (YYYY-MM-DD), 'yesterday' or 'yesterday-N', set ARKAD_FROM and ARKAD_TO in that form";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_use_summer_time_in_new_york_when_the_date_is_in_july() {
        let from = Some(OsString::from("yesterday"));
        let to = None;
        // 00:30 on 1 July in New York (UTC-4). A fixed UTC-5 offset gives 23:30 on 30 June.
        let now = DateTime::parse_from_rfc3339("2026-07-01T04:30:00Z")
            .expect("Given a hardcoded RFC 3339 time, parsing should always succeed")
            .with_timezone(&Utc);

        let expected_result = Ok(vec![date("2026-06-30")]);

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_use_winter_time_in_new_york_when_the_date_is_in_january() {
        let from = Some(OsString::from("yesterday"));
        let to = None;
        // 23:30 on 14 January in New York (UTC-5). A fixed UTC-4 offset gives 00:30 on 15 January.
        let now = DateTime::parse_from_rfc3339("2026-01-15T04:30:00Z")
            .expect("Given a hardcoded RFC 3339 time, parsing should always succeed")
            .with_timezone(&Utc);

        let expected_result = Ok(vec![date("2026-01-13")]);

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_when_the_month_and_the_day_have_one_digit() {
        let from = Some(OsString::from("2026-1-5"));
        let to = None;
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::InvalidDate {
            variable: "ARKAD_FROM",
            value: "2026-1-5".to_string(),
        });

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_when_the_year_has_a_sign() {
        // The one-digit month keeps the text at 10 characters, so a length check does not catch it.
        let from = Some(OsString::from("+2026-1-05"));
        let to = None;
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::InvalidDate {
            variable: "ARKAD_FROM",
            value: "+2026-1-05".to_string(),
        });

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_last_date_and_today_when_the_last_date_is_not_past() {
        let error = InvalidRunDates::InvalidLastDate {
            to: date("2026-11-01"),
            today: date("2026-11-01"),
        };

        let expected_result = "[InvalidLastDate] ARKAD_TO date '2026-11-01' is not before today '2026-11-01' in New York, the SEC publishes a daily index only after its day ends";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_first_and_the_last_date_when_the_dates_are_out_of_order() {
        let error = InvalidRunDates::InvalidDateOrder {
            from: date("2026-11-01"),
            to: date("2026-10-31"),
        };

        let expected_result = "[InvalidDateOrder] ARKAD_FROM date '2026-11-01' comes after the last date '2026-10-31'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_last_four_days_when_from_is_yesterday_minus_three() {
        let from = Some(OsString::from("yesterday-3"));
        let to = None;
        let now = noon_utc();

        let expected_result = Ok(vec![
            date("2026-10-28"),
            date("2026-10-29"),
            date("2026-10-30"),
            date("2026-10-31"),
        ]);

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_day_before_yesterday_when_to_is_yesterday_minus_one() {
        let from = Some(OsString::from("2026-10-30"));
        let to = Some(OsString::from("yesterday-1"));
        let now = noon_utc();

        let expected_result = Ok(vec![date("2026-10-30")]);

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_when_yesterday_minus_has_zero_days() {
        let from = Some(OsString::from("yesterday-0"));
        let to = None;
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::InvalidDate {
            variable: "ARKAD_FROM",
            value: "yesterday-0".to_string(),
        });

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_when_yesterday_minus_has_a_leading_zero() {
        let from = Some(OsString::from("yesterday-03"));
        let to = None;
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::InvalidDate {
            variable: "ARKAD_FROM",
            value: "yesterday-03".to_string(),
        });

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_when_yesterday_minus_has_a_sign() {
        let from = Some(OsString::from("yesterday-+3"));
        let to = None;
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::InvalidDate {
            variable: "ARKAD_FROM",
            value: "yesterday-+3".to_string(),
        });

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_when_yesterday_minus_has_no_number() {
        let from = Some(OsString::from("yesterday-"));
        let to = None;
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::InvalidDate {
            variable: "ARKAD_FROM",
            value: "yesterday-".to_string(),
        });

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_date_when_yesterday_minus_goes_before_the_earliest_supported_date() {
        let from = Some(OsString::from("yesterday-100000000"));
        let to = None;
        let now = noon_utc();

        let expected_result = Err(InvalidRunDates::InvalidDate {
            variable: "ARKAD_FROM",
            value: "yesterday-100000000".to_string(),
        });

        let result = run_dates(from, to, now);

        assert_eq!(result, expected_result);
    }
}
