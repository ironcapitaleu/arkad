//! # Period
//!
//! Time period representations for financial data points.
//! Balance sheet items use [`Period::Instant`], income statement and
//! cash flow items use [`Period::Duration`].

use std::fmt;

use chrono::NaiveDate;

/// The time period a financial data point covers.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Period {
    /// A point-in-time measurement (e.g., balance sheet date).
    Instant {
        /// The measurement date.
        date: NaiveDate,
    },
    /// A measurement over a time range (e.g., revenue for a quarter).
    Duration {
        /// The start of the period (inclusive).
        start: NaiveDate,
        /// The end of the period (inclusive).
        end: NaiveDate,
    },
}

impl fmt::Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instant { date } => write!(f, "{date}"),
            Self::Duration { start, end } => write!(f, "{start} to {end}"),
        }
    }
}
