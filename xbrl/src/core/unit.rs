//! # Unit
//!
//! Measurement units for financial data points.

use std::fmt;

/// The unit of measurement for a financial data point.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Unit {
    /// United States Dollars.
    Usd,
    /// Number of shares.
    Shares,
    /// Dollars per share (e.g., EPS).
    UsdPerShare,
    /// Dimensionless ratio or percentage.
    Pure,
}

impl Unit {
    /// Parses a [`Unit`] from an SEC XBRL unit string.
    ///
    /// Returns `None` if the input does not match any known unit.
    #[must_use]
    pub fn from_sec_str(s: &str) -> Option<Self> {
        match s {
            "USD" => Some(Self::Usd),
            "shares" => Some(Self::Shares),
            "USD/shares" | "USD/share" => Some(Self::UsdPerShare),
            "pure" => Some(Self::Pure),
            _ => None,
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usd => write!(f, "USD"),
            Self::Shares => write!(f, "shares"),
            Self::UsdPerShare => write!(f, "USD/share"),
            Self::Pure => write!(f, "pure"),
        }
    }
}
