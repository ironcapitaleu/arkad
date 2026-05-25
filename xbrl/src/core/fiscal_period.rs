//! # Fiscal Period
//!
//! Fiscal period identifiers as reported in SEC filings.

use std::fmt;

/// The fiscal period a data point was reported for.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum FiscalPeriod {
    /// Full fiscal year.
    Fy,
    /// First fiscal quarter.
    Q1,
    /// Second fiscal quarter.
    Q2,
    /// Third fiscal quarter.
    Q3,
    /// Fourth fiscal quarter.
    Q4,
}

impl FiscalPeriod {
    /// Parses a [`FiscalPeriod`] from an SEC fiscal period string.
    ///
    /// Returns `None` if the input does not match any known period.
    #[must_use]
    pub fn from_sec_str(s: &str) -> Option<Self> {
        match s {
            "FY" => Some(Self::Fy),
            "Q1" => Some(Self::Q1),
            "Q2" => Some(Self::Q2),
            "Q3" => Some(Self::Q3),
            "Q4" => Some(Self::Q4),
            _ => None,
        }
    }
}

impl fmt::Display for FiscalPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fy => write!(f, "FY"),
            Self::Q1 => write!(f, "Q1"),
            Self::Q2 => write!(f, "Q2"),
            Self::Q3 => write!(f, "Q3"),
            Self::Q4 => write!(f, "Q4"),
        }
    }
}
