//! # Fiscal Period
//!
//! Provides the [`FiscalPeriod`] enum representing a company's fiscal reporting period.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

/// A company's fiscal reporting period — a full year or one of its fiscal quarters.
///
/// Relative to the company's fiscal year, which need not align with the calendar (e.g. Apple's
/// fiscal year ends in September, so its fiscal Q4 falls in calendar Q3).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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

impl Display for FiscalPeriod {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Fy => write!(f, "FY"),
            Self::Q1 => write!(f, "Q1"),
            Self::Q2 => write!(f, "Q2"),
            Self::Q3 => write!(f, "Q3"),
            Self::Q4 => write!(f, "Q4"),
        }
    }
}

impl FiscalPeriod {
    /// Parses a [`FiscalPeriod`] from its SEC string (e.g. `"FY"`, `"Q1"`), or `None` if unrecognized.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::financial::fiscal_period::FiscalPeriod;
    ///
    /// assert_eq!(FiscalPeriod::from_sec_str("Q1"), Some(FiscalPeriod::Q1));
    /// assert_eq!(FiscalPeriod::from_sec_str("H1"), None);
    /// ```
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_display_fy_when_fiscal_period_is_full_year() {
        let expected_result = "FY";

        let result = FiscalPeriod::Fy.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_q1_from_sec_string_when_input_is_q1() {
        let expected_result = Some(FiscalPeriod::Q1);

        let result = FiscalPeriod::from_sec_str("Q1");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_sec_string_is_unknown_period() {
        let expected_result = None;

        let result = FiscalPeriod::from_sec_str("H1");

        assert_eq!(result, expected_result);
    }
}
