//! # Period
//!
//! Provides [`Period`], the time a fact covers: an instant or a duration.

use std::fmt;

use chrono::NaiveDate;

use super::invalid_value::{InvalidValue, InvalidValueReason};
use super::taxonomy::reference::PeriodType;

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

    /// Returns the exact start, or `None` for an instant.
    #[must_use]
    pub const fn start(&self) -> Option<NaiveDate> {
        match self {
            Self::Instant { .. } => None,
            Self::Duration(duration) => Some(duration.start),
        }
    }

    /// Returns the period type: [`PeriodType::Instant`] or [`PeriodType::Duration`].
    #[must_use]
    pub const fn period_type(&self) -> PeriodType {
        match self {
            Self::Instant { .. } => PeriodType::Instant,
            Self::Duration(_) => PeriodType::Duration,
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

/// The days by which two period ends can differ and still name the same period. A 52-53 week
/// fiscal year ends up to 6 days away from the end of its month.
pub(super) const TOLERANCE_DAYS: i64 = 7;

/// Returns `true` if `left` and `right` lie at most [`TOLERANCE_DAYS`] days apart.
pub(super) fn is_near(left: NaiveDate, right: NaiveDate) -> bool {
    (left - right).num_days().abs() <= TOLERANCE_DAYS
}

/// A quarter is `QUARTER_NUMERATOR / QUARTER_DENOMINATOR` days, that is 91.3125 days.
const QUARTER_NUMERATOR: i64 = 1461;

/// The denominator of the length of a quarter in days. See [`QUARTER_NUMERATOR`].
const QUARTER_DENOMINATOR: i64 = 16;

/// Returns the days of `quarters` quarters, rounded down.
pub(crate) fn days_of_quarters(quarters: u16) -> i64 {
    i64::from(quarters) * QUARTER_NUMERATOR / QUARTER_DENOMINATOR
}

/// A span of time: an exact start, an exact end, and a length in quarters.
///
/// [`Duration::from_dates`] computes the length from the start and the end, so the two cannot
/// disagree.
///
/// The length needs 16 bits: Notes `qtrs` reaches 4,007.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Duration {
    /// The exact start.
    start: NaiveDate,
    /// The exact end.
    end: NaiveDate,
    /// The length in quarters. It is a field, because the value key and the series key use it.
    quarters: u16,
}

impl Duration {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "Duration";

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
                Self::TYPE_NAME,
                InvalidValueReason::EndBeforeStart,
                format!("{start} to {end}"),
            ));
        }
        // 91.3125 days are 1461/16 days, so `round(days / 91.3125)` is `round(16 * days / 1461)`.
        // 1461 is odd, so the fraction never lies halfway, and integer division rounds exactly.
        let quarters = (QUARTER_DENOMINATOR * days + QUARTER_NUMERATOR / 2) / QUARTER_NUMERATOR;
        let quarters = u16::try_from(quarters).map_err(|_| {
            InvalidValue::new(
                Self::TYPE_NAME,
                InvalidValueReason::OutOfRange,
                format!("{start} to {end}"),
            )
        })?;
        Ok(Self {
            start,
            end,
            quarters,
        })
    }

    /// Returns how far the length lies from its whole number of quarters, in sixteenths of a day:
    /// `|16 * days - 1461 * quarters|`.
    ///
    /// The SEC orders the copies of one key by this distance, so a 91-day quarter comes before a
    /// 121-day period that also rounds to 1 quarter.
    #[must_use]
    pub fn distance_to_full_quarters(&self) -> i64 {
        let days = (self.end - self.start).num_days();
        (QUARTER_DENOMINATOR * days - QUARTER_NUMERATOR * i64::from(self.quarters)).abs()
    }

    /// Returns the exact start.
    #[must_use]
    pub const fn start(&self) -> NaiveDate {
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
        write!(f, "{} to {}", self.start, self.end)
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
    fn should_give_the_period_type_for_an_instant_and_a_duration() {
        let periods = [
            Period::Instant {
                end: date(2026, 9, 30),
            },
            Period::Duration(
                Duration::from_dates(date(2026, 7, 1), date(2026, 9, 30))
                    .expect("Given an end after the start, the creation should always succeed"),
            ),
        ];

        let expected_result = vec![PeriodType::Instant, PeriodType::Duration];

        let result: Vec<PeriodType> = periods.iter().map(Period::period_type).collect();

        assert_eq!(result, expected_result);
    }
    #[test]
    const fn should_implement_auto_traits_for_period() {
        implements_auto_traits::<Period>();
    }

    #[test]
    const fn should_implement_auto_traits_for_duration() {
        implements_auto_traits::<Duration>();
    }

    #[test]
    fn should_give_the_distance_to_a_full_quarter_for_a_four_month_stub() {
        let duration = Duration::from_dates(date(2026, 6, 1), date(2026, 9, 30))
            .expect("Given an end after the start, the creation should always succeed");

        let expected_result = 475;

        let result = duration.distance_to_full_quarters();

        assert_eq!(result, expected_result);
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
    fn should_give_the_exact_start_when_the_period_is_a_duration() {
        let period = Period::Duration(
            Duration::from_dates(date(2026, 6, 1), date(2026, 9, 30))
                .expect("Given an end after the start, the creation should always succeed"),
        );

        let expected_result = Some(date(2026, 6, 1));

        let result = period.start();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_exact_start_when_the_period_is_an_instant() {
        let period = Period::Instant {
            end: date(2026, 9, 30),
        };

        let expected_result = None;

        let result = period.start();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_four_thousand_quarters_when_notes_reports_them() {
        let start = date(1022, 6, 30);
        let end = date(2024, 3, 31);

        let expected_result = 4007;

        let result = Duration::from_dates(start, end)
            .expect("Given an end after the start, the creation should always succeed")
            .quarters();

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
        let period = Period::Duration(
            Duration::from_dates(date(2023, 10, 1), date(2024, 9, 28))
                .expect("Given an end after the start, the creation should always succeed"),
        );

        let expected_result = date(2024, 9, 28);

        let result = period.end();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_start_and_end_when_period_is_a_duration() {
        let duration = Duration::from_dates(date(2023, 10, 1), date(2024, 9, 28))
            .expect("Given an end after the start, the creation should always succeed");

        let expected_result = "2023-10-01 to 2024-09-28";

        let result = duration.to_string();

        assert_eq!(result, expected_result);
    }
}
