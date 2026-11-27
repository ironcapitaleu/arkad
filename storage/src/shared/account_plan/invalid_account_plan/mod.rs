//! # Invalid Account Plan
//!
//! Provides [`InvalidAccountPlan`], the error raised when a string is not an account plan.

use thiserror::Error;

/// Error raised when a string is not the stored name of a plan, so it cannot be an account plan.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[InvalidAccountPlan] Account plan '{value}' is not 'free', 'screener_pro', 'api_developer', or 'api_business'"
)]
pub struct InvalidAccountPlan {
    /// The string that was rejected.
    pub value: String,
}

impl InvalidAccountPlan {
    /// Creates a new [`InvalidAccountPlan`] for the rejected string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_invalid_account_plan() {
        implements_auto_traits::<InvalidAccountPlan>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_account_plan() {
        implements_value_traits::<InvalidAccountPlan>();
    }

    #[test]
    fn should_name_the_rejected_string_when_displaying_invalid_account_plan() {
        let error = InvalidAccountPlan::new("pro");

        let expected_result = "[InvalidAccountPlan] Account plan 'pro' is not 'free', 'screener_pro', 'api_developer', or 'api_business'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
