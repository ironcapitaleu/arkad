//! # Period
//!
//! Provides the [`Period`] enum representing the time window of a financial observation.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use chrono::NaiveDate;

/// The time window of a financial observation.
///
/// SEC XBRL data measures values either at a single point or over a range, and the two carry
/// different fields — so they are modeled as distinct variants rather than a date pair with
/// optional start:
///
/// - `Instant`: a snapshot on one date (Balance Sheet items, e.g. total assets on Sep 30, 2023).
/// - `Duration`: a measurement spanning a range (Income Statement / Cash Flow items).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

impl Display for Period {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
