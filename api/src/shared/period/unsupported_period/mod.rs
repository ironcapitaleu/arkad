//! # Unsupported Period
//!
//! Provides the [`UnsupportedPeriod`] error for a kind of period that the `/v1` contract has no
//! shape for.

use thiserror::Error;

/// Error indicating that the `/v1` contract has no shape for a kind of period.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("[UnsupportedPeriod] The contract has no shape for the period '{period}'")]
pub struct UnsupportedPeriod {
    /// The period as the domain prints it.
    pub period: String,
}

impl UnsupportedPeriod {
    /// Creates a new [`UnsupportedPeriod`] error for `period`.
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
    const fn should_implement_auto_traits_for_unsupported_period() {
        implements_auto_traits::<UnsupportedPeriod>();
    }

    #[test]
    fn should_name_the_period_when_displaying_unsupported_period() {
        let error = UnsupportedPeriod::new("2024-09-28");

        let expected_result =
            "[UnsupportedPeriod] The contract has no shape for the period '2024-09-28'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
