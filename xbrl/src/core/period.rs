//! # Period
//!
//! Provides [`Period`], the time a fact covers: an instant or a duration.

use std::fmt;

use chrono::NaiveDate;

use super::invalid_value::{InvalidValue, InvalidValueReason};

/// The time a fact covers.
///
/// A balance sheet fact covers an [`Period::Instant`]. An income statement or cash flow fact covers
/// a [`Period::Duration`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Period {
    /// A point in time, for example the date of a balance sheet.
    Instant {
        /// The date of the instant.
        end: NaiveDate,
    },
    /// A span of time, for example a fiscal quarter.
    Duration(Duration),
}

impl Period {
    /// Returns the end date. For an instant, the end is the date of the instant.
    #[must_use]
    pub const fn end(&self) -> NaiveDate {
        match self {
            Self::Instant { end } => *end,
            Self::Duration(duration) => duration.end,
        }
    }

    /// Returns the length in quarters. The value key holds this length. An instant has 0.
    #[must_use]
    pub const fn quarters(&self) -> u16 {
        match self {
            Self::Instant { .. } => 0,
            Self::Duration(duration) => duration.quarters,
        }
    }
}

impl fmt::Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instant { end } => write!(f, "{end}"),
            Self::Duration(duration) => write!(f, "{duration}"),
        }
    }
}

/// A span of time: an exact end, a length in quarters, and the exact start if the source gives it.
///
/// The Notes data set gives the length but often no exact start. So the length is a field, and the
/// start is optional. If the start is known, [`Duration::from_dates`] computes the length, so the
/// two cannot disagree.
///
/// The length needs 16 bits: Notes `qtrs` reaches 4,007.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Duration {
    start: Option<NaiveDate>,
    end: NaiveDate,
    quarters: u16,
}

impl Duration {
    /// Creates a [`Duration`] from an exact start and end, and computes its length in quarters.
    ///
    /// The length is `round(days(end - start) / 91.3125)`, the rule that matched Notes `qtrs` on
    /// the 622 filings that both the Notes data set and the instance documents hold.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] with [`InvalidValueReason::EndBeforeStart`] if `end` lies before
    /// `start`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use xbrl::core::period::Duration;
    ///
    /// let start = NaiveDate::from_ymd_opt(2023, 10, 1).expect("Given a hardcoded valid date, the creation should always succeed");
    /// let end = NaiveDate::from_ymd_opt(2024, 9, 28).expect("Given a hardcoded valid date, the creation should always succeed");
    ///
    /// let expected_result = 4;
    ///
    /// let duration = Duration::from_dates(start, end).expect("Given an end after the start, the creation should always succeed");
    /// let result = duration.quarters();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn from_dates(start: NaiveDate, end: NaiveDate) -> Result<Self, InvalidValue> {
        let days = (end - start).num_days();
        if days < 0 {
            return Err(InvalidValue::new(
                "Duration",
                InvalidValueReason::EndBeforeStart,
                format!("{start} to {end}"),
            ));
        }
        // 91.3125 days are 1461/16 days, so `round(days / 91.3125)` is `round(16 * days / 1461)`.
        // 1461 is odd, so the fraction never lies halfway, and integer division rounds exactly.
        let quarters = (16 * days + 730) / 1461;
        let quarters = u16::try_from(quarters).map_err(|_| {
            InvalidValue::new(
                "Duration",
                InvalidValueReason::OutOfRange,
                format!("{start} to {end}"),
            )
        })?;
        Ok(Self {
            start: Some(start),
            end,
            quarters,
        })
    }

    /// Creates a [`Duration`] from an exact end and a length in quarters, for a source with no
    /// exact start, such as the Notes data set.
    #[must_use]
    pub const fn from_end_and_quarters(end: NaiveDate, quarters: u16) -> Self {
        Self {
            start: None,
            end,
            quarters,
        }
    }

    /// Returns the exact start, if the source gives it.
    #[must_use]
    pub const fn start(&self) -> Option<NaiveDate> {
        self.start
    }

    /// Returns the exact end.
    #[must_use]
    pub const fn end(&self) -> NaiveDate {
        self.end
    }

    /// Returns the length in quarters.
    #[must_use]
    pub const fn quarters(&self) -> u16 {
        self.quarters
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.start {
            Some(start) => write!(f, "{start} to {}", self.end),
            None => write!(f, "{} quarters to {}", self.quarters, self.end),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded valid date, the creation should always succeed")
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_period() {
        implements_auto_traits::<Period>();
    }

    #[test]
    const fn should_implement_auto_traits_for_duration() {
        implements_auto_traits::<Duration>();
    }

    #[test]
    fn should_count_one_quarter_when_duration_spans_thirteen_weeks() {
        let start = date(2024, 6, 30);
        let end = date(2024, 9, 28);

        let expected_result = 1;

        let result = Duration::from_dates(start, end)
            .expect("Given an end after the start, the creation should always succeed")
            .quarters();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_three_quarters_when_duration_spans_nine_months() {
        let start = date(2024, 1, 1);
        let end = date(2024, 9, 30);

        let expected_result = 3;

        let result = Duration::from_dates(start, end)
            .expect("Given an end after the start, the creation should always succeed")
            .quarters();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_zero_quarters_when_duration_spans_one_day() {
        let start = date(2024, 9, 27);
        let end = date(2024, 9, 28);

        let expected_result = 0;

        let result = Duration::from_dates(start, end)
            .expect("Given an end after the start, the creation should always succeed")
            .quarters();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_duration_when_end_lies_before_start() {
        let start = date(2024, 9, 28);
        let end = date(2024, 6, 30);

        let expected_result = Err(InvalidValue::new(
            "Duration",
            InvalidValueReason::EndBeforeStart,
            "2024-09-28 to 2024-06-30",
        ));

        let result = Duration::from_dates(start, end);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_duration_when_quarters_exceed_sixteen_bits() {
        let start = date(1, 1, 1);
        let end = date(20_000, 1, 1);

        let expected_result = Err(InvalidValue::new(
            "Duration",
            InvalidValueReason::OutOfRange,
            "0001-01-01 to +20000-01-01",
        ));

        let result = Duration::from_dates(start, end);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_no_start_when_duration_comes_from_end_and_quarters() {
        let end = date(2024, 9, 28);

        let expected_result = None;

        let result = Duration::from_end_and_quarters(end, 4).start();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_four_thousand_quarters_when_notes_reports_them() {
        let end = date(2024, 3, 31);
        let quarters = 4007;

        let expected_result = 4007;

        let result = Period::Duration(Duration::from_end_and_quarters(end, quarters)).quarters();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_zero_quarters_when_period_is_an_instant() {
        let period = Period::Instant {
            end: date(2024, 9, 28),
        };

        let expected_result = 0;

        let result = period.quarters();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_end_when_period_is_a_duration() {
        let period = Period::Duration(Duration::from_end_and_quarters(date(2024, 9, 28), 4));

        let expected_result = date(2024, 9, 28);

        let result = period.end();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_start_and_end_when_duration_has_a_start() {
        let duration = Duration::from_dates(date(2023, 10, 1), date(2024, 9, 28))
            .expect("Given an end after the start, the creation should always succeed");

        let expected_result = "2023-10-01 to 2024-09-28";

        let result = duration.to_string();

        assert_eq!(result, expected_result);
    }
}
