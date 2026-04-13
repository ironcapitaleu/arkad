//! # Period Module
//!
//! Provides the [`Period`] enum representing the time window for a financial observation.
//! SEC XBRL data distinguishes between instant measurements (Balance Sheet items at a point in time)
//! and duration measurements (Income Statement/Cash Flow items over a range).

use std::fmt;

use chrono::NaiveDate;

/// Time period for a financial observation.
///
/// Distinguishes between point-in-time snapshots and measurements over a date range.
///
/// - **Instant**: A snapshot at a specific date (e.g., "Total Assets on Sep 30, 2023").
/// - **Duration**: A measurement over a period (e.g., "Revenue from Oct 2022 through Sep 2023").
///
/// # Example
/// ```
/// use chrono::NaiveDate;
/// use sec::shared::financial::period::Period;
///
/// let instant = Period::Instant {
///     date: NaiveDate::from_ymd_opt(2023, 9, 30).unwrap(),
/// };
///
/// let duration = Period::Duration {
///     start: NaiveDate::from_ymd_opt(2022, 10, 1).unwrap(),
///     end: NaiveDate::from_ymd_opt(2023, 9, 30).unwrap(),
/// };
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Period {
    /// A point-in-time snapshot (e.g., Balance Sheet items).
    Instant {
        /// The date of the measurement.
        date: NaiveDate,
    },
    /// A measurement over a date range (e.g., Income Statement items).
    Duration {
        /// The start of the measurement period (inclusive).
        start: NaiveDate,
        /// The end of the measurement period (inclusive).
        end: NaiveDate,
    },
}

impl fmt::Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instant { date } => write!(f, "Instant({date})"),
            Self::Duration { start, end } => write!(f, "Duration({start} to {end})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_display_instant_with_date_when_period_is_instant() {
        let period = Period::Instant {
            date: NaiveDate::from_ymd_opt(2023, 9, 30)
                .expect("Hardcoded date should always be valid"),
        };

        let expected_result = "Instant(2023-09-30)";

        let result = period.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_duration_with_range_when_period_is_duration() {
        let period = Period::Duration {
            start: NaiveDate::from_ymd_opt(2022, 10, 1)
                .expect("Hardcoded date should always be valid"),
            end: NaiveDate::from_ymd_opt(2023, 9, 30)
                .expect("Hardcoded date should always be valid"),
        };

        let expected_result = "Duration(2022-10-01 to 2023-09-30)";

        let result = period.to_string();

        assert_eq!(result, expected_result);
    }
}
