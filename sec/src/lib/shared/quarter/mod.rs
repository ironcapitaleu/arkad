//! # Quarter
//!
//! Provides [`Quarter`], one calendar quarter of one year. It names the folder of an EDGAR daily
//! index or full index, as in `full-index/2026/QTR4/`.

use std::fmt::{self, Display, Formatter};

use chrono::{Datelike, NaiveDate};
use serde::{Serialize, Serializer};

/// The quarter folders of the daily index and the full index, in the order of the quarters.
const QUARTER_FOLDERS: [&str; 4] = ["QTR1", "QTR2", "QTR3", "QTR4"];

/// The number of months in one quarter.
const MONTHS_PER_QUARTER: u32 = 3;

/// The number of quarters in one year.
const QUARTERS_PER_YEAR: u32 = 4;

/// One calendar quarter of one year, for example the fourth quarter of 2026.
///
/// Quarters order by year, then by their place in the year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quarter {
    year: i32,
    /// The place of the quarter in its year, from 0 to 3.
    index: u32,
}

impl Quarter {
    /// Returns the quarter that holds `date`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use sec::shared::quarter::Quarter;
    ///
    /// let date = NaiveDate::from_ymd_opt(2026, 10, 1)
    ///     .expect("Given a hardcoded calendar date, the date should always build");
    ///
    /// let expected_result = "2026/QTR4";
    ///
    /// let result = Quarter::of(date).to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn of(date: NaiveDate) -> Self {
        Self {
            year: date.year(),
            index: date.month0() / MONTHS_PER_QUARTER,
        }
    }

    /// Returns the year of the quarter, the `YYYY` part of the URL.
    #[must_use]
    pub const fn year(&self) -> i32 {
        self.year
    }

    /// Returns the folder of the quarter, `QTR1` to `QTR4`, the `QTRn` part of the URL.
    #[must_use]
    pub const fn folder(&self) -> &'static str {
        QUARTER_FOLDERS[self.index as usize]
    }

    /// Returns the quarter after this one. The quarter after the fourth is the first of the next
    /// year.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use sec::shared::quarter::Quarter;
    ///
    /// let date = NaiveDate::from_ymd_opt(2026, 12, 31)
    ///     .expect("Given a hardcoded calendar date, the date should always build");
    ///
    /// let expected_result = "2027/QTR1";
    ///
    /// let result = Quarter::of(date).next().to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn next(&self) -> Self {
        if self.index + 1 == QUARTERS_PER_YEAR {
            Self {
                year: self.year + 1,
                index: 0,
            }
        } else {
            Self {
                year: self.year,
                index: self.index + 1,
            }
        }
    }

    /// Returns the first day of the quarter, or `None` if the quarter lies after the last year that
    /// [`NaiveDate`] holds.
    #[must_use]
    pub const fn first_day(&self) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(self.year, self.index * MONTHS_PER_QUARTER + 1, 1)
    }
}

impl Display for Quarter {
    /// Writes the year and the folder as the SEC writes them in a path, for example `2026/QTR4`.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.year, self.folder())
    }
}

/// Serializes as the [`Display`] text, for example `"2026/QTR4"`.
impl Serialize for Quarter {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, the date should always build")
    }

    #[test]
    fn should_return_the_first_quarter_when_the_date_is_the_first_day_of_the_year() {
        let date = date(2026, 1, 1);

        let expected_result = "QTR1";

        let result = Quarter::of(date).folder();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_first_quarter_when_the_date_is_the_last_day_of_march() {
        let date = date(2026, 3, 31);

        let expected_result = "QTR1";

        let result = Quarter::of(date).folder();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_second_quarter_when_the_date_is_the_first_day_of_april() {
        let date = date(2026, 4, 1);

        let expected_result = "QTR2";

        let result = Quarter::of(date).folder();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_third_quarter_when_the_date_is_the_last_day_of_september() {
        let date = date(2026, 9, 30);

        let expected_result = "QTR3";

        let result = Quarter::of(date).folder();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_fourth_quarter_when_the_date_is_the_last_day_of_the_year() {
        let date = date(2026, 12, 31);

        let expected_result = "QTR4";

        let result = Quarter::of(date).folder();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_year_of_the_date_for_quarter() {
        let date = date(2026, 10, 1);

        let expected_result = 2026;

        let result = Quarter::of(date).year();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_equal_when_two_dates_lie_in_one_quarter() {
        let first_date = date(2026, 10, 1);
        let last_date = date(2026, 12, 31);

        let expected_result = Quarter::of(first_date);

        let result = Quarter::of(last_date);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_the_fourth_quarter_before_the_first_quarter_of_the_next_year() {
        let earlier = Quarter::of(date(2026, 12, 31));
        let later = Quarter::of(date(2027, 1, 1));

        let expected_result = true;

        let result = earlier < later;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_next_quarter_of_the_same_year_when_the_quarter_is_the_third() {
        let quarter = Quarter::of(date(2026, 9, 30));

        let expected_result = Quarter::of(date(2026, 10, 1));

        let result = quarter.next();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_first_quarter_of_the_next_year_when_the_quarter_is_the_fourth() {
        let quarter = Quarter::of(date(2026, 12, 31));

        let expected_result = Quarter::of(date(2027, 1, 1));

        let result = quarter.next();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_first_day_of_the_quarter_when_the_quarter_holds_a_date_in_its_middle() {
        let quarter = Quarter::of(date(2026, 8, 15));

        let expected_result = Some(date(2026, 7, 1));

        let result = quarter.first_day();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_first_day_when_the_quarter_lies_after_the_last_year_of_naive_date() {
        let quarter = Quarter::of(NaiveDate::MAX);

        let expected_result = None;

        let result = quarter.next().first_day();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_as_the_year_and_the_folder_for_quarter() {
        let quarter = Quarter::of(date(2026, 7, 15));

        let expected_result = serde_json::json!("2026/QTR3");

        let result = serde_json::to_value(quarter)
            .expect("Given a quarter, serialization to JSON should always succeed");

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_quarter() {
        implements_auto_traits::<Quarter>();
    }
}
