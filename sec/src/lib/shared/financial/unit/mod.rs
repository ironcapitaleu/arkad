//! # Unit
//!
//! Provides the [`Unit`] enum representing the measurement unit of a financial data point.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

/// The measurement unit of a financial data point, as expected for its XBRL concept.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Usd => write!(f, "USD"),
            Self::Shares => write!(f, "shares"),
            Self::UsdPerShare => write!(f, "USD/share"),
            Self::Pure => write!(f, "pure"),
        }
    }
}

impl Unit {
    /// Parses a [`Unit`] from its SEC XBRL unit string (e.g. `"USD"`), or `None` if unrecognized.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::financial::unit::Unit;
    ///
    /// assert_eq!(Unit::from_sec_str("USD"), Some(Unit::Usd));
    /// assert_eq!(Unit::from_sec_str("unknown_unit"), None);
    /// ```
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
