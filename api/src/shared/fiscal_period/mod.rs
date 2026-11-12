//! # Fiscal Period
//!
//! Provides the [`FiscalPeriod`] of a fact or a filing in the format of the `/v1` contract.
//!
//! ## Modules
//!
//! - [`unsupported_fiscal_period`]: The [`UnsupportedFiscalPeriod`] error for a period that the
//!   contract does not name.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use xbrl::core::fiscal_period::FiscalPeriod as DomainFiscalPeriod;

use crate::traits::from_domain::FromDomain;

pub mod unsupported_fiscal_period;

pub use unsupported_fiscal_period::UnsupportedFiscalPeriod;

/// The fiscal period of a fact or a filing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FiscalPeriod {
    /// The full fiscal year.
    Fy,
    /// The first quarter of the fiscal year.
    Q1,
    /// The second quarter of the fiscal year.
    Q2,
    /// The third quarter of the fiscal year.
    Q3,
    /// The fourth quarter of the fiscal year.
    Q4,
}

impl TryFrom<DomainFiscalPeriod> for FiscalPeriod {
    type Error = UnsupportedFiscalPeriod;

    /// Converts an `xbrl` [`FiscalPeriod`](DomainFiscalPeriod) into a contract [`FiscalPeriod`].
    ///
    /// The match ignores case, so `fy` and `FY` both convert.
    ///
    /// # Errors
    ///
    /// Returns an [`UnsupportedFiscalPeriod`] if the period is not `FY` or `Q1` to `Q4`, for
    /// example the half year `H1`.
    fn try_from(period: DomainFiscalPeriod) -> Result<Self, Self::Error> {
        match period.as_str().to_ascii_uppercase().as_str() {
            "FY" => Ok(Self::Fy),
            "Q1" => Ok(Self::Q1),
            "Q2" => Ok(Self::Q2),
            "Q3" => Ok(Self::Q3),
            "Q4" => Ok(Self::Q4),
            _ => Err(UnsupportedFiscalPeriod::new(period.as_str())),
        }
    }
}

impl FromDomain<DomainFiscalPeriod> for FiscalPeriod {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_fiscal_period() {
        implements_auto_traits::<FiscalPeriod>();
    }

    fn domain_period(text: &str) -> DomainFiscalPeriod {
        DomainFiscalPeriod::new(text).expect(
            "Given a hardcoded period without whitespace, the creation should always succeed",
        )
    }

    #[test]
    fn should_convert_to_fy_when_the_domain_period_is_fy() {
        let domain = domain_period("FY");

        let expected_result = Ok(FiscalPeriod::Fy);

        let result = FiscalPeriod::try_from(domain);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_with_unsupported_fiscal_period_when_the_domain_period_is_a_half_year() {
        let domain = domain_period("H1");

        let expected_result = Err(UnsupportedFiscalPeriod::new("H1"));

        let result = FiscalPeriod::try_from(domain);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_in_lowercase_for_q1() {
        let period = FiscalPeriod::Q1;

        let expected_result = json!("q1");

        let result = serde_json::to_value(period)
            .expect("Given a unit variant, serializing it should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_fy_when_the_domain_period_is_lowercase() {
        let domain = domain_period("fy");

        let expected_result = Ok(FiscalPeriod::Fy);

        let result = FiscalPeriod::try_from(domain);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_q1_when_the_query_holds_lowercase_q1() {
        let text = json!("q1");

        let expected_result = FiscalPeriod::Q1;

        let result: FiscalPeriod = serde_json::from_value(text).expect(
            "Given a lowercase period that the contract names, reading it should always succeed",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_value_when_the_query_holds_uppercase_q1() {
        let text = json!("Q1");

        let expected_result = true;

        // `serde_json::Error` has no `PartialEq`, so the test compares the check.
        let result = serde_json::from_value::<FiscalPeriod>(text).is_err();

        assert_eq!(result, expected_result);
    }
}
