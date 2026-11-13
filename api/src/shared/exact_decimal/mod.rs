//! # Exact Decimal
//!
//! Provides the [`ExactDecimal`] value of a fact in the format of the `/v1` contract.

use bigdecimal::BigDecimal;
use serde::Serialize;
use utoipa::ToSchema;

use crate::traits::from_domain::FromDomain;

/// An exact decimal number as a string without an exponent, for example `391035000000`.
///
/// A JSON number passes through a 64-bit float in many clients, and a float cannot hold every
/// decimal exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[schema(example = "391035000000")]
pub struct ExactDecimal(String);

impl ExactDecimal {
    /// Returns the number as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<BigDecimal> for ExactDecimal {
    /// Converts a [`BigDecimal`] into an [`ExactDecimal`].
    fn from(value: BigDecimal) -> Self {
        Self(value.to_plain_string())
    }
}

impl FromDomain<BigDecimal> for ExactDecimal {}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_exact_decimal() {
        implements_auto_traits::<ExactDecimal>();
    }

    #[test]
    fn should_write_no_exponent_when_the_value_has_a_negative_scale() {
        let value = BigDecimal::from_str("3.91035E+11")
            .expect("Given a hardcoded decimal text, parsing it should always succeed");

        let expected_result = "391035000000";

        let decimal = ExactDecimal::from(value);
        let result = decimal.as_str();

        assert_eq!(result, expected_result);
    }
}
