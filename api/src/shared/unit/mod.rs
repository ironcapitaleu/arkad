//! # Unit
//!
//! Provides the [`Unit`] of a fact in the format of the `/v1` contract.

use serde::Serialize;
use utoipa::ToSchema;
use xbrl::core::unit::{Measure, Unit as DomainUnit};

use crate::traits::from_domain::FromDomain;

/// The unit of a fact as a string: an ISO 4217 currency code, `shares`, `usd_per_share`, or
/// `pure` for a ratio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[schema(example = "USD")]
pub struct Unit(String);

impl Unit {
    /// Returns the unit as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<DomainUnit> for Unit {
    /// Converts an `xbrl` [`Unit`](DomainUnit) into a contract [`Unit`].
    ///
    /// A currency per share becomes `<currency>_per_share` in lowercase, for example
    /// `usd_per_share`. Every other unit keeps the text of the `xbrl` unit.
    fn from(unit: DomainUnit) -> Self {
        let text = match (unit.numerator(), unit.denominator()) {
            ([currency], [shares]) if shares.as_str() == "shares" && is_currency(currency) => {
                format!("{}_per_share", currency.as_str().to_ascii_lowercase())
            }
            _ => unit.to_string(),
        };
        Self(text)
    }
}

impl FromDomain<DomainUnit> for Unit {}

fn is_currency(measure: &Measure) -> bool {
    DomainUnit::new(vec![measure.clone()], Vec::new()).is_ok_and(|unit| unit.is_currency())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_unit() {
        implements_auto_traits::<Unit>();
    }

    fn domain_unit(text: &str) -> DomainUnit {
        DomainUnit::from_str(text)
            .expect("Given a hardcoded valid unit text, parsing it should always succeed")
    }

    #[test]
    fn should_keep_the_currency_code_when_the_domain_unit_is_usd() {
        let domain = domain_unit("USD");

        let expected_result = "USD";

        let unit = Unit::from(domain);
        let result = unit.as_str();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_usd_per_share_when_the_domain_unit_is_usd_per_shares() {
        let domain = domain_unit("USD/shares");

        let expected_result = "usd_per_share";

        let unit = Unit::from(domain);
        let result = unit.as_str();

        assert_eq!(result, expected_result);
    }
}
