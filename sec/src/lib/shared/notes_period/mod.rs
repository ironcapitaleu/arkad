//! # Notes Period
//!
//! Provides the [`NotesPeriod`] type for the period of one SEC Financial Statement and Notes file.
//!
//! The SEC publishes one Notes file per quarter from 2009q1 on. For the latest quarters, it
//! publishes one file per month instead, and about a year after the quarter ends it merges the
//! three monthly files into one quarterly file. A [`NotesPeriod`] holds a quarter or a month from
//! 2009 on, and gives the SEC file name of the period. The SEC page that links the Notes files
//! says which periods exist now, see
//! [`NotesFileList`](crate::shared::notes_file_list::NotesFileList).
//!
//! ## Modules
//!
//! - [`constants`]: The first year and the [`FILE_NAME_OVERRIDES`].
//! - [`notes_period_error`]: The [`NotesPeriodError`] returned when validation fails, and its
//!   [`InvalidNotesPeriodReason`].
//! - [`month`]: The [`Month`] type for a month of a calendar year.
//! - [`quarter`]: The [`Quarter`] type for a quarter of a calendar year.
//! - [`year`]: The [`Year`] type for a calendar year.

pub mod constants;
pub mod month;
pub mod notes_period_error;
pub mod quarter;
pub mod year;

pub use constants::FILE_NAME_OVERRIDES;
pub use month::Month;
pub use notes_period_error::{InvalidNotesPeriodReason, NotesPeriodError};
pub use quarter::Quarter;
pub use year::Year;

use std::fmt::{self, Display, Formatter};

use chrono::{Months, NaiveDate};
use serde::{Serialize, Serializer};

use constants::{FILE_NAME_SUFFIX, FIRST_YEAR};

/// The period of one SEC Financial Statement and Notes file: a quarter or a month of a year.
///
/// Values are only produced through [`NotesPeriod::new`] (or the [`TryFrom`] impl). A period has
/// no upper limit, so a period after the latest published file also parses. The type does not
/// check whether the SEC publishes a file for the period. The SEC page says that.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotesPeriod {
    year: Year,
    part: PeriodPart,
}

/// The part of the year that one Notes file covers.
///
/// The variant order puts every quarter of a year before every month of the same year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum PeriodPart {
    Quarter(Quarter),
    Month(Month),
}

impl NotesPeriod {
    /// Validates a string of the form `YYYYqN` or `YYYY_MM` into a [`NotesPeriod`].
    ///
    /// Trims surrounding whitespace before validation.
    ///
    /// # Errors
    ///
    /// Returns a [`NotesPeriodError`] if the input matches neither form
    /// ([`InvalidNotesPeriodReason::InvalidNotesPeriodFormat`]), or the year comes before 2009
    /// ([`InvalidNotesPeriodReason::UnpublishedYear`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::notes_period::NotesPeriod;
    ///
    /// let period = NotesPeriod::new(" 2025_10 ").expect("A hardcoded published month should always parse");
    ///
    /// let expected_result = "2025_10";
    ///
    /// let result = period.to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn new(period: &str) -> Result<Self, NotesPeriodError> {
        let error = |reason| NotesPeriodError::new(reason, period);
        let (year, part) = parse(period.trim())
            .ok_or_else(|| error(InvalidNotesPeriodReason::InvalidNotesPeriodFormat))?;

        if year < FIRST_YEAR {
            return Err(error(InvalidNotesPeriodReason::UnpublishedYear { year }));
        }
        Ok(Self { year, part })
    }

    /// Returns the name of the SEC file that holds the Notes of this period.
    ///
    /// The name is the period plus `_notes.zip`, except for the periods in
    /// [`FILE_NAME_OVERRIDES`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::notes_period::NotesPeriod;
    ///
    /// let period = NotesPeriod::new("2010q1").expect("A hardcoded published quarter should always parse");
    ///
    /// let expected_result = "2010q1_notes_1.zip";
    ///
    /// let result = period.file_name();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn file_name(&self) -> String {
        let period = self.to_string();
        FILE_NAME_OVERRIDES
            .iter()
            .find(|(overridden_period, _)| *overridden_period == period)
            .map_or_else(
                || format!("{period}{FILE_NAME_SUFFIX}"),
                |(_, file_name)| (*file_name).to_string(),
            )
    }
    /// Returns the last calendar day of the period: the last day of its quarter or of its month.
    ///
    /// # Panics
    ///
    /// Never panics: every year of a [`NotesPeriod`] fits a [`NaiveDate`].
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use sec::shared::notes_period::NotesPeriod;
    ///
    /// let period = NotesPeriod::new("2024q1").expect("A hardcoded published quarter should always parse");
    ///
    /// let expected_result =
    ///     NaiveDate::from_ymd_opt(2024, 3, 31).expect("A hardcoded calendar date should always be valid");
    ///
    /// let result = period.last_day();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn last_day(&self) -> NaiveDate {
        let last_month = match self.part {
            PeriodPart::Quarter(Quarter::Q1) => 3,
            PeriodPart::Quarter(Quarter::Q2) => 6,
            PeriodPart::Quarter(Quarter::Q3) => 9,
            PeriodPart::Quarter(Quarter::Q4) => 12,
            PeriodPart::Month(month) => month.0.number_from_month(),
        };
        NaiveDate::from_ymd_opt(i32::from(self.year.0), last_month, 1)
            .and_then(|first_day| first_day.checked_add_months(Months::new(1)))
            .and_then(|first_day_of_next_month| first_day_of_next_month.pred_opt())
            .expect("Given a year of at most 65535 and a month from 1 to 12, the last day of the month should always exist")
    }
}

/// Splits a trimmed period string into its year and part, or `None` if it has neither form.
fn parse(period: &str) -> Option<(Year, PeriodPart)> {
    if let Some((year, quarter)) = period.split_once('q') {
        let quarter = Quarter::from_number_str(quarter)?;
        return Some((Year::from_number_str(year)?, PeriodPart::Quarter(quarter)));
    }
    let (year, month) = period.split_once('_')?;
    let month = Month::from_number_str(month)?;
    Some((Year::from_number_str(year)?, PeriodPart::Month(month)))
}

/// Returns whether `number` is exactly `length` ASCII digits.
fn is_ascii_digits(number: &str, length: usize) -> bool {
    number.len() == length && number.bytes().all(|byte| byte.is_ascii_digit())
}

impl Display for NotesPeriod {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.part {
            PeriodPart::Quarter(quarter) => {
                write!(f, "{}{quarter}", self.year)
            }
            PeriodPart::Month(month) => {
                write!(f, "{}_{month}", self.year)
            }
        }
    }
}

impl Serialize for NotesPeriod {
    /// Serializes the period as the SEC writes it, for example `"2024q1"`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl TryFrom<&str> for NotesPeriod {
    type Error = NotesPeriodError;

    /// Delegates to [`NotesPeriod::new`].
    ///
    /// # Errors
    ///
    /// Returns a [`NotesPeriodError`] in the same cases as [`NotesPeriod::new`].
    fn try_from(period: &str) -> Result<Self, Self::Error> {
        Self::new(period)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    /// The 79 file names linked on the SEC page, measured on 2026-10-28.
    const MEASURED_FILE_NAMES: [&str; 79] = [
        "2009q1_notes.zip",
        "2009q2_notes.zip",
        "2009q3_notes.zip",
        "2009q4_notes.zip",
        "2010q1_notes_1.zip",
        "2010q2_notes_0.zip",
        "2010q3_notes_0.zip",
        "2010q4_notes.zip",
        "2011q1_notes.zip",
        "2011q2_notes.zip",
        "2011q3_notes.zip",
        "2011q4_notes.zip",
        "2012q1_notes.zip",
        "2012q2_notes.zip",
        "2012q3_notes.zip",
        "2012q4_notes.zip",
        "2013q1_notes.zip",
        "2013q2_notes.zip",
        "2013q3_notes.zip",
        "2013q4_notes.zip",
        "2014q1_notes.zip",
        "2014q2_notes.zip",
        "2014q3_notes.zip",
        "2014q4_notes.zip",
        "2015q1_notes.zip",
        "2015q2_notes.zip",
        "2015q3_notes.zip",
        "2015q4_notes.zip",
        "2016q1_notes.zip",
        "2016q2_notes.zip",
        "2016q3_notes.zip",
        "2016q4_notes.zip",
        "2017q1_notes.zip",
        "2017q2_notes.zip",
        "2017q3_notes.zip",
        "2017q4_notes.zip",
        "2018q1_notes.zip",
        "2018q2_notes.zip",
        "2018q3_notes.zip",
        "2018q4_notes.zip",
        "2019q1_notes.zip",
        "2019q2_notes.zip",
        "2019q3_notes.zip",
        "2019q4_notes.zip",
        "2020q1_notes.zip",
        "2020q2_notes.zip",
        "2020q3_notes.zip",
        "2020q4_notes.zip",
        "2021q1_notes.zip",
        "2021q2_notes.zip",
        "2021q3_notes.zip",
        "2021q4_notes.zip",
        "2022q1_notes.zip",
        "2022q2_notes.zip",
        "2022q3_notes.zip",
        "2022q4_notes.zip",
        "2023q1_notes.zip",
        "2023q2_notes.zip",
        "2023q3_notes.zip",
        "2023q4_notes.zip",
        "2024q1_notes.zip",
        "2024q2_notes.zip",
        "2024q3_notes.zip",
        "2024q4_notes.zip",
        "2025q1_notes.zip",
        "2025q2_notes.zip",
        "2025q3_notes.zip",
        "2025_10_notes.zip",
        "2025_11_notes.zip",
        "2025_12_notes.zip",
        "2026_01_notes.zip",
        "2026_02_notes.zip",
        "2026_03_notes.zip",
        "2026_04_notes.zip",
        "2026_05_notes.zip",
        "2026_06_notes.zip",
        "2026_07_notes.zip",
        "2026_08_notes.zip",
        "2026_09_notes.zip",
    ];

    fn rejection_reason(period: &str) -> Option<InvalidNotesPeriodReason> {
        NotesPeriod::new(period).err().map(|error| error.reason)
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_notes_period() {
        implements_auto_traits::<NotesPeriod>();
    }

    const fn implements_value_traits<T: Debug + Copy + PartialEq + Eq + PartialOrd + Ord + Hash>() {
    }
    #[test]
    const fn should_implement_value_traits_for_notes_period() {
        implements_value_traits::<NotesPeriod>();
    }

    #[test]
    fn should_parse_year_and_quarter_when_input_has_the_quarter_form() {
        let period_string = "2024q1";

        let expected_result = Ok(NotesPeriod {
            year: Year(2024),
            part: PeriodPart::Quarter(Quarter::Q1),
        });

        let result = NotesPeriod::new(period_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_year_and_month_when_input_has_the_month_form() {
        let period_string = "2025_10";

        let expected_result = Ok(NotesPeriod {
            year: Year(2025),
            part: PeriodPart::Month(Month(chrono::Month::October)),
        });

        let result = NotesPeriod::new(period_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_the_first_quarter_when_input_is_2009q1() {
        let period_string = "2009q1";

        let expected_result = None;

        let result = rejection_reason(period_string);

        assert_eq!(result, expected_result);
    }

    // The SEC merges the months of 2025q4 into one quarterly file in about January 2027. A fixed
    // last quarter of 2025q3 rejected that file.
    #[test]
    fn should_accept_a_quarter_when_quarter_is_after_2025q3() {
        let period_string = "2025q4";

        let expected_result = None;

        let result = rejection_reason(period_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_a_month_when_month_is_before_2025_10() {
        let period_string = "2025_09";

        let expected_result = None;

        let result = rejection_reason(period_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_with_unpublished_year_when_year_is_before_2009() {
        let period_string = "2008q4";

        let expected_result = Some(InvalidNotesPeriodReason::UnpublishedYear { year: Year(2008) });

        let result = rejection_reason(period_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_with_unpublished_year_when_period_is_a_month_before_2009() {
        let period_string = "2008_12";

        let expected_result = Some(InvalidNotesPeriodReason::UnpublishedYear { year: Year(2008) });

        let result = rejection_reason(period_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_a_month_when_month_is_after_the_last_published_file() {
        let period_string = "2099_12";

        let expected_result = None;

        let result = NotesPeriod::new(period_string).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_the_period_when_input_has_surrounding_whitespace() {
        let period_string = " \t2024q1\n";

        let expected_result = NotesPeriod::new("2024q1");

        let result = NotesPeriod::new(period_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_the_same_period_when_try_from_is_called() {
        let period_string = "2025_10";

        let expected_result = NotesPeriod::new(period_string);

        let result = NotesPeriod::try_from(period_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_with_invalid_format_when_input_matches_neither_form() {
        let malformed_periods = [
            "", "2024", "2024Q1", "2024q5", "2024q01", "24q1", "2025_1", "2025_13", "2025_00",
            "2025-10", "+025_10",
        ];

        let expected_result =
            vec![Some(InvalidNotesPeriodReason::InvalidNotesPeriodFormat); malformed_periods.len()];

        let result: Vec<_> = malformed_periods
            .into_iter()
            .map(rejection_reason)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_a_quarter_before_a_month_of_a_later_quarter_for_notes_periods() {
        let quarter = NotesPeriod::new("2025q3")
            .expect("Given a hardcoded valid period, parsing should always succeed");
        let month = NotesPeriod::new("2025_10")
            .expect("Given a hardcoded valid period, parsing should always succeed");

        let expected_result = true;

        let result = quarter < month;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_sec_form_when_period_is_a_quarter() {
        let period = NotesPeriod::new("2024q1")
            .expect("Given a published quarter, parsing should always succeed");

        let expected_result = "2024q1";

        let result = period.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_a_zero_padded_month_when_period_is_a_month() {
        let period = NotesPeriod::new("2026_01")
            .expect("Given a published month, parsing should always succeed");

        let expected_result = "2026_01";

        let result = period.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_sec_form_when_period_is_a_month() {
        let period = NotesPeriod::new("2025_10")
            .expect("Given the first published month, parsing should always succeed");

        let expected_result = "\"2025_10\"";

        let result = serde_json::to_string(&period)
            .expect("Serializing a period as a JSON string should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_measured_file_names_for_all_periods_from_2009q1_to_2026_09() {
        let quarters = (2009..=2025)
            .flat_map(|year| (1..=4).map(move |quarter| format!("{year}q{quarter}")))
            .take(67);
        let months = (10..=12)
            .map(|month| format!("2025_{month}"))
            .chain((1..=9).map(|month| format!("2026_0{month}")));

        let expected_result = MEASURED_FILE_NAMES.to_vec();

        let result: Vec<String> = quarters
            .chain(months)
            .map(|period| {
                NotesPeriod::new(&period)
                    .expect("Given a period from 2009q1 to 2026_09, parsing should always succeed")
                    .file_name()
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, building it should always succeed")
    }

    fn period(period: &str) -> NotesPeriod {
        NotesPeriod::new(period)
            .expect("Given a hardcoded published period, parsing should always succeed")
    }

    #[test]
    fn should_return_the_last_day_of_the_quarter_when_period_is_a_quarter() {
        let period = period("2009q2");

        let expected_result = date(2009, 6, 30);

        let result = period.last_day();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_last_day_of_february_when_period_is_a_leap_year_february() {
        let period = period("2028_02");

        let expected_result = date(2028, 2, 29);

        let result = period.last_day();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_last_day_of_the_year_when_period_is_a_december() {
        let period = period("2025_12");

        let expected_result = date(2025, 12, 31);

        let result = period.last_day();

        assert_eq!(result, expected_result);
    }
}
