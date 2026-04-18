//! # Fiscal Year Module
//!
//! Provides the [`FiscalYear`] newtype wrapping a fiscal year number.
//! Prevents bare `u16` values from being confused with other numeric fields.

use std::fmt;

/// A fiscal year identifier (e.g., 2023, 2024).
///
/// Wraps the `fy` field from SEC XBRL filing data.
///
/// # Example
/// ```
/// use sec::shared::financial::fiscal_year::FiscalYear;
///
/// let fy = FiscalYear::from(2023_u16);
/// assert_eq!(fy.value(), 2023);
/// assert_eq!(fy.to_string(), "2023");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct FiscalYear(u16);

impl FiscalYear {
    /// Returns the fiscal year as a `u16`.
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl From<u16> for FiscalYear {
    fn from(year: u16) -> Self {
        Self(year)
    }
}

impl TryFrom<i32> for FiscalYear {
    type Error = std::num::TryFromIntError;

    fn try_from(year: i32) -> Result<Self, Self::Error> {
        u16::try_from(year).map(Self)
    }
}

impl TryFrom<u32> for FiscalYear {
    type Error = std::num::TryFromIntError;

    fn try_from(year: u32) -> Result<Self, Self::Error> {
        u16::try_from(year).map(Self)
    }
}

impl TryFrom<i64> for FiscalYear {
    type Error = std::num::TryFromIntError;

    fn try_from(year: i64) -> Result<Self, Self::Error> {
        u16::try_from(year).map(Self)
    }
}

impl TryFrom<u64> for FiscalYear {
    type Error = std::num::TryFromIntError;

    fn try_from(year: u64) -> Result<Self, Self::Error> {
        u16::try_from(year).map(Self)
    }
}

impl fmt::Display for FiscalYear {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_create_fiscal_year_from_u16() {
        let expected_result = 2023;

        let result = FiscalYear::from(2023_u16).value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_fiscal_year_from_u32_when_value_fits_in_u16() {
        let expected_result = 2024;

        let result = FiscalYear::try_from(2024_u32)
            .expect("2024 fits in u16")
            .value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_fiscal_year_from_i32_when_value_is_positive_and_fits_in_u16() {
        let expected_result = 2025;

        let result = FiscalYear::try_from(2025_i32)
            .expect("2025 fits in u16")
            .value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_create_fiscal_year_from_i32_when_value_is_negative() {
        let expected_result = true;

        let result = FiscalYear::try_from(-1_i32).is_err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_fiscal_year_from_u64_when_value_fits_in_u16() {
        let expected_result = 2023;

        let result = FiscalYear::try_from(2023_u64)
            .expect("2023 fits in u16")
            .value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_fiscal_year_from_i64_when_value_is_positive_and_fits_in_u16() {
        let expected_result = 2026;

        let result = FiscalYear::try_from(2026_i64)
            .expect("2026 fits in u16")
            .value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_create_fiscal_year_from_u32_when_value_exceeds_u16() {
        let expected_result = true;

        let result = FiscalYear::try_from(100_000_u32).is_err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_create_fiscal_year_from_u64_when_value_exceeds_u16() {
        let expected_result = true;

        let result = FiscalYear::try_from(100_000_u64).is_err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_create_fiscal_year_from_i64_when_value_is_negative() {
        let expected_result = true;

        let result = FiscalYear::try_from(-2024_i64).is_err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_year_when_formatted() {
        let fy = FiscalYear::from(2024_u16);

        let expected_result = "2024";

        let result = fy.to_string();

        assert_eq!(result, expected_result);
    }
}
