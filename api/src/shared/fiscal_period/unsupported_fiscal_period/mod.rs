//! # Unsupported Fiscal Period
//!
//! Provides the [`UnsupportedFiscalPeriod`] error for a fiscal period that the `/v1` contract
//! does not name.

use thiserror::Error;

/// Error indicating that the `/v1` contract does not name a fiscal period.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("[UnsupportedFiscalPeriod] The contract does not name the fiscal period '{period}'")]
pub struct UnsupportedFiscalPeriod {
    /// The fiscal period as the domain writes it, for example `H1`.
    pub period: String,
}

impl UnsupportedFiscalPeriod {
    /// Creates a new [`UnsupportedFiscalPeriod`] error for `period`.
    #[must_use]
    pub fn new(period: impl Into<String>) -> Self {
        Self {
            period: period.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_unsupported_fiscal_period() {
        implements_auto_traits::<UnsupportedFiscalPeriod>();
    }

    #[test]
    fn should_name_the_period_when_displaying_unsupported_fiscal_period() {
        let error = UnsupportedFiscalPeriod::new("H1");

        let expected_result =
            "[UnsupportedFiscalPeriod] The contract does not name the fiscal period 'H1'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
