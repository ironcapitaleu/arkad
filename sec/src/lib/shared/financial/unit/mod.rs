//! # Unit Module
//!
//! Provides the [`Unit`] enum representing measurement units for SEC financial data.
//! Each XBRL concept has an expected unit type (e.g., monetary values in USD,
//! share counts in shares).

use std::fmt;

/// Measurement unit for a financial data point.
///
/// Represents the unit of measurement associated with an XBRL concept value.
/// Uses `#[non_exhaustive]` to allow future extensions without breaking changes.
///
/// # Example
/// ```
/// use sec::shared::financial::unit::Unit;
///
/// let unit = Unit::Usd;
/// assert_eq!(unit.to_string(), "USD");
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Unit {
    /// United States Dollars.
    Usd,
    /// Share count.
    Shares,
    /// US Dollars per share (e.g., earnings per share).
    UsdPerShare,
    /// Dimensionless ratio or percentage.
    Pure,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Usd => write!(f, "USD"),
            Self::Shares => write!(f, "shares"),
            Self::UsdPerShare => write!(f, "USD/share"),
            Self::Pure => write!(f, "pure"),
        }
    }
}

impl Unit {
    /// Attempts to parse a [`Unit`] from an SEC XBRL unit string.
    ///
    /// # Errors
    ///
    /// Returns `None` if the input string does not match any known unit.
    #[must_use]
    pub fn from_sec_str(s: &str) -> Option<Self> {
        match s {
            "USD" => Some(Self::Usd),
            "shares" => Some(Self::Shares),
            "USD/shares" => Some(Self::UsdPerShare),
            "pure" => Some(Self::Pure),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_display_usd_when_unit_is_usd() {
        let unit = Unit::Usd;

        let expected_result = "USD";

        let result = unit.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_usd_from_sec_string_when_input_is_usd() {
        let expected_result = Some(Unit::Usd);

        let result = Unit::from_sec_str("USD");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_shares_from_sec_string_when_input_is_shares() {
        let expected_result = Some(Unit::Shares);

        let result = Unit::from_sec_str("shares");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_sec_string_is_unknown() {
        let expected_result = None;

        let result = Unit::from_sec_str("unknown_unit");

        assert_eq!(result, expected_result);
    }
}
