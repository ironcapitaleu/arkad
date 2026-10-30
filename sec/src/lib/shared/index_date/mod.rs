//! # Index Date
//!
//! Provides the [`IndexDate`] type for the date of one EDGAR daily index.
//!
//! The SEC publishes the daily index of a date at
//! `https://www.sec.gov/Archives/edgar/daily-index/YYYY/QTRn/form.YYYYMMDD.idx`. An [`IndexDate`]
//! gives the three date parts of that URL: [`IndexDate::year`], [`IndexDate::quarter`], and
//! [`IndexDate::basic_format`].
//!
//! ## Modules
//!
//! - [`constants`]: The [`FIRST_INDEX_DATE`] and the quarter folders.
//! - [`index_date_error`]: The [`IndexDateError`] returned when validation fails, and its
//!   [`InvalidIndexDateReason`].
//! - [`listing`]: The parser for the dates in the listing of a quarter folder.

use std::fmt::{self, Display, Formatter};

use chrono::{Datelike, NaiveDate, Utc};
use serde::{Serialize, Serializer};

use constants::QUARTER_FOLDERS;

pub mod constants;
pub mod index_date_error;
pub mod listing;

pub use constants::FIRST_INDEX_DATE;
pub use index_date_error::{IndexDateError, InvalidIndexDateReason};

/// The date of one EDGAR daily index.
///
/// Only [`IndexDate::new`] and the [`TryFrom`] impl create values. The date is never before
/// [`FIRST_INDEX_DATE`], and never after the current UTC date at the time of validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndexDate(NaiveDate);

impl IndexDate {
    /// Validates a string of the form `YYYY-MM-DD` into an [`IndexDate`].
    ///
    /// Trims surrounding whitespace before validation. Today is the current UTC date.
    ///
    /// # Errors
    ///
    /// Returns an [`IndexDateError`] if the input is not a calendar date of the form `YYYY-MM-DD`
    /// ([`InvalidIndexDateReason::InvalidIndexDateFormat`]), the date is before
    /// [`FIRST_INDEX_DATE`] ([`InvalidIndexDateReason::EarlyIndexDate`]), or the date is after
    /// today ([`InvalidIndexDateReason::FutureIndexDate`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::index_date::IndexDate;
    ///
    /// let date = IndexDate::new(" 2026-10-27 ").expect("A hardcoded past date should always parse");
    ///
    /// let expected_result = "2026-10-27";
    ///
    /// let result = date.to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn new(date: &str) -> Result<Self, IndexDateError> {
        validate(date, Utc::now().date_naive())
    }

    /// Returns the year of the date, the `YYYY` part of the URL.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::index_date::IndexDate;
    ///
    /// let date = IndexDate::new("2026-10-27").expect("A hardcoded past date should always parse");
    ///
    /// let expected_result = 2026;
    ///
    /// let result = date.year();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn year(&self) -> i32 {
        self.0.year()
    }

    /// Returns the quarter folder of the date, `QTR1` to `QTR4`, the `QTRn` part of the URL.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::index_date::IndexDate;
    ///
    /// let date = IndexDate::new("2026-10-27").expect("A hardcoded past date should always parse");
    ///
    /// let expected_result = "QTR4";
    ///
    /// let result = date.quarter();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn quarter(&self) -> &'static str {
        QUARTER_FOLDERS[self.0.month0() as usize / 3]
    }

    /// Returns the date in the ISO 8601 basic format `YYYYMMDD`, the date part of the file name.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::index_date::IndexDate;
    ///
    /// let date = IndexDate::new("2026-03-02").expect("A hardcoded past date should always parse");
    ///
    /// let expected_result = "20260302";
    ///
    /// let result = date.basic_format();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn basic_format(&self) -> String {
        self.0.format("%Y%m%d").to_string()
    }

    /// Returns the date as a [`NaiveDate`].
    #[must_use]
    pub const fn date(&self) -> NaiveDate {
        self.0
    }

    /// Returns the first day of the next quarter, or `None` if that day is after the current UTC
    /// date.
    #[must_use]
    pub fn first_day_of_next_quarter(&self) -> Option<Self> {
        first_day_of_next_quarter(self.0, Utc::now().date_naive())
    }
}

/// Validates a date string against `today`.
fn validate(date: &str, today: NaiveDate) -> Result<IndexDate, IndexDateError> {
    let error = |reason| IndexDateError::new(reason, date);
    let parsed =
        parse(date.trim()).ok_or_else(|| error(InvalidIndexDateReason::InvalidIndexDateFormat))?;
    if parsed < FIRST_INDEX_DATE {
        return Err(error(InvalidIndexDateReason::EarlyIndexDate));
    }
    if parsed > today {
        return Err(error(InvalidIndexDateReason::FutureIndexDate { today }));
    }
    Ok(IndexDate(parsed))
}

/// Parses a trimmed `YYYY-MM-DD` string, or returns `None` if it has another form or names no
/// calendar date.
///
/// Checks the shape first, because chrono also accepts a sign or a single-digit month or day.
fn parse(date: &str) -> Option<NaiveDate> {
    let is_well_formed = date.len() == 10
        && date.bytes().enumerate().all(|(index, byte)| match index {
            4 | 7 => byte == b'-',
            _ => byte.is_ascii_digit(),
        });
    if !is_well_formed {
        return None;
    }
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

/// Returns the first day of the quarter after `date`, or `None` if that day is after `today`.
fn first_day_of_next_quarter(date: NaiveDate, today: NaiveDate) -> Option<IndexDate> {
    let next_quarter_month = date.month0() / 3 * 3 + 4;
    let next_quarter_start = if next_quarter_month > 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(date.year(), next_quarter_month, 1)
    }?;
    (next_quarter_start <= today).then_some(IndexDate(next_quarter_start))
}

impl Display for IndexDate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

impl Serialize for IndexDate {
    /// Serializes the date in the form `YYYY-MM-DD`, for example `"2026-10-27"`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl TryFrom<&str> for IndexDate {
    type Error = IndexDateError;

    /// Delegates to [`IndexDate::new`].
    ///
    /// # Errors
    ///
    /// Returns an [`IndexDateError`] in the same cases as [`IndexDate::new`].
    fn try_from(date: &str) -> Result<Self, Self::Error> {
        Self::new(date)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{Debug, Display};
    use std::hash::Hash;

    use pretty_assertions::assert_eq;
    use serde::Serialize;

    use super::*;

    /// A fixed today, so that the tests do not depend on the clock.
    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 30)
            .expect("Given a hardcoded valid date, construction should always succeed")
    }

    fn index_date(date: &str) -> IndexDate {
        validate(date, today())
            .expect("Given a hardcoded date before today, validation should always succeed")
    }

    fn quarters(dates: [&str; 2]) -> [&'static str; 2] {
        dates.map(|date| index_date(date).quarter())
    }

    fn rejection_reason(date: &str) -> Option<InvalidIndexDateReason> {
        validate(date, today()).err().map(|error| error.reason)
    }

    const fn implements_request_input_traits<
        T: Debug + Display + Clone + Eq + Ord + Hash + Serialize + Send + Sync + Unpin,
    >() {
    }
    #[test]
    const fn should_implement_request_input_traits_for_index_date() {
        implements_request_input_traits::<IndexDate>();
    }

    #[test]
    fn should_return_the_first_day_of_the_next_quarter_when_it_is_not_after_today() {
        let expected_result = Some(index_date("2026-07-01"));

        let result = first_day_of_next_quarter(index_date("2026-05-15").0, today());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_january_first_of_the_next_year_for_a_date_in_qtr4() {
        let expected_result = Some(index_date("2026-01-01"));

        let result = first_day_of_next_quarter(index_date("2025-12-31").0, today());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_first_day_of_the_next_quarter_when_it_is_today() {
        let expected_result = Some(index_date("2026-10-01"));

        let result =
            first_day_of_next_quarter(index_date("2026-09-30").0, index_date("2026-10-01").0);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_the_next_quarter_starts_after_today() {
        let expected_result = None;

        let result = first_day_of_next_quarter(index_date("2026-10-27").0, today());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_the_date_when_input_has_the_form_yyyy_mm_dd() {
        let expected_result = Ok(IndexDate(
            NaiveDate::from_ymd_opt(2026, 10, 27)
                .expect("Given a hardcoded valid date, construction should always succeed"),
        ));

        let result = validate("2026-10-27", today());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_the_date_when_input_has_surrounding_whitespace() {
        let expected_result = validate("2026-10-27", today());

        let result = validate(" \t2026-10-27\n", today());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_the_date_when_date_is_today() {
        let expected_result = Ok(IndexDate(today()));

        let result = validate("2026-10-30", today());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_with_future_index_date_when_date_is_after_today() {
        let expected_result = Some(InvalidIndexDateReason::FutureIndexDate { today: today() });

        let result = rejection_reason("2026-10-31");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_with_future_index_date_when_new_gets_a_date_in_year_9999() {
        let expected_result = true;

        let result = matches!(
            IndexDate::new("9999-12-31").map_err(|error| error.reason),
            Err(InvalidIndexDateReason::FutureIndexDate { .. })
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_with_invalid_format_when_input_is_not_a_yyyy_mm_dd_date() {
        let malformed_dates = [
            "",
            "2026",
            "20261027",
            "2026-1-27",
            "2026-10-7",
            "26-10-27",
            "2026/10/27",
            "+026-10-27",
            "2026-10-27T00:00",
            "２6-10-27",
            "2026-13-01",
            "2026-00-10",
            "2026-04-31",
            "2025-02-29",
        ];

        let expected_result =
            vec![Some(InvalidIndexDateReason::InvalidIndexDateFormat); malformed_dates.len()];

        let result: Vec<_> = malformed_dates.into_iter().map(rejection_reason).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_the_date_when_date_is_the_first_index_date() {
        let expected_result = Ok(IndexDate(FIRST_INDEX_DATE));

        let result = validate("1998-05-18", today());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_with_early_index_date_when_date_is_before_the_first_index_date() {
        let expected_result = Some(InvalidIndexDateReason::EarlyIndexDate);

        let result = rejection_reason("1998-05-17");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_with_early_index_date_when_year_is_0000() {
        let expected_result = Some(InvalidIndexDateReason::EarlyIndexDate);

        let result = rejection_reason("0000-01-01");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_february_29_when_year_is_a_leap_year() {
        let expected_result = true;

        let result = validate("2024-02-29", today()).is_ok();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_switch_from_qtr1_to_qtr2_when_date_crosses_april_1() {
        let expected_result = ["QTR1", "QTR2"];

        let result = quarters(["2026-03-31", "2026-04-01"]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_switch_from_qtr2_to_qtr3_when_date_crosses_july_1() {
        let expected_result = ["QTR2", "QTR3"];

        let result = quarters(["2026-06-30", "2026-07-01"]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_switch_from_qtr3_to_qtr4_when_date_crosses_october_1() {
        let expected_result = ["QTR3", "QTR4"];

        let result = quarters(["2026-09-30", "2026-10-01"]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_switch_from_qtr4_to_qtr1_when_date_crosses_january_1() {
        let expected_result = ["QTR4", "QTR1"];

        let result = quarters(["2025-12-31", "2026-01-01"]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_year_when_date_is_on_new_years_eve() {
        let expected_result = 2025;

        let result = index_date("2025-12-31").year();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_zero_padded_basic_format_when_month_and_day_have_one_digit() {
        let expected_result = "20260102";

        let result = index_date("2026-01-02").basic_format();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_yyyy_mm_dd_form_for_index_dates() {
        let expected_result = "\"2026-01-02\"";

        let result = serde_json::to_string(&index_date("2026-01-02"))
            .expect("Serializing a date as a JSON string should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_earlier_date_first_for_index_dates() {
        let expected_result = true;

        let result = index_date("2025-12-31") < index_date("2026-01-01");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_the_same_date_when_try_from_is_called() {
        let expected_result = IndexDate::new("2026-10-27");

        let result = IndexDate::try_from("2026-10-27");

        assert_eq!(result, expected_result);
    }
}
