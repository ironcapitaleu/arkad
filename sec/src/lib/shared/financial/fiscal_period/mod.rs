//! # Fiscal Period Module
//!
//! Provides the [`FiscalPeriod`] enum representing a company's fiscal reporting period.
//! Unlike calendar quarters, fiscal periods are relative to a company's fiscal year
//! (e.g., Apple's FY ends in September, so their fiscal Q4 is calendar Q3).

use std::fmt;

/// Fiscal reporting period.
///
/// Represents the fiscal period associated with a financial data point.
/// A company's fiscal year may not align with the calendar year.
///
/// # Example
/// ```
/// use sec::shared::financial::fiscal_period::FiscalPeriod;
///
/// let period = FiscalPeriod::Fy;
/// assert_eq!(period.to_string(), "FY");
/// ```
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

impl FiscalPeriod {
    /// Attempts to parse a [`FiscalPeriod`] from an SEC fiscal period string.
    ///
    /// # Errors
    ///
    /// Returns `None` if the input string does not match any known fiscal period.
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
