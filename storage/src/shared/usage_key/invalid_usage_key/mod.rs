//! # Invalid Usage Key
//!
//! Provides [`InvalidUsageKey`], the error raised when a string is not a usage key, and
//! [`InvalidUsageKeyReason`], the part that failed.

use std::fmt::{self, Display, Formatter};

use thiserror::Error;

/// Error raised when a string is not `<account_id>:<channel>:<day>`, so it cannot be a usage key.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[InvalidUsageKey] Usage key '{value}' is not '<account_id>:<channel>:<day>', Reason: '{reason}'"
)]
pub struct InvalidUsageKey {
    /// The part of the string that failed.
    pub reason: InvalidUsageKeyReason,
    /// The string that was rejected.
    pub value: String,
}

impl InvalidUsageKey {
    /// Creates a new [`InvalidUsageKey`] for the rejected string and the part that failed.
    #[must_use]
    pub fn new(reason: InvalidUsageKeyReason, value: impl Into<String>) -> Self {
        Self {
            reason,
            value: value.into(),
        }
    }
}

/// Enum representing the reason why a string is not a usage key.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidUsageKeyReason {
    /// The string does not split into three parts at the colons.
    WrongPartCount {
        /// The number of parts the string splits into.
        count: usize,
    },
    /// The first part is not a lowercase hyphenated UUID.
    InvalidAccountId,
    /// The second part is not `api_key` or `login`.
    InvalidChannel,
    /// The third part is not a calendar date in the form `YYYY-MM-DD`, such as `2026-11-26`.
    InvalidDay,
}

impl Display for InvalidUsageKeyReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongPartCount { count } => write!(f, "the string has {count} parts, not 3"),
            Self::InvalidAccountId => {
                f.write_str("the first part is not a lowercase hyphenated UUID")
            }
            Self::InvalidChannel => f.write_str("the second part is not a channel"),
            Self::InvalidDay => {
                f.write_str("the third part is not a calendar date in the form YYYY-MM-DD")
            }
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
    const fn should_implement_auto_traits_for_invalid_usage_key() {
        implements_auto_traits::<InvalidUsageKey>();
    }

    #[test]
    const fn should_implement_auto_traits_for_invalid_usage_key_reason() {
        implements_auto_traits::<InvalidUsageKeyReason>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_usage_key() {
        implements_value_traits::<InvalidUsageKey>();
    }

    #[test]
    const fn should_implement_value_traits_for_invalid_usage_key_reason() {
        implements_value_traits::<InvalidUsageKeyReason>();
    }

    #[test]
    fn should_name_the_string_and_the_reason_when_displaying_invalid_usage_key() {
        let error = InvalidUsageKey::new(InvalidUsageKeyReason::InvalidDay, "a:login:b");

        let expected_result = "[InvalidUsageKey] Usage key 'a:login:b' is not '<account_id>:<channel>:<day>', Reason: 'the third part is not a calendar date in the form YYYY-MM-DD'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_part_count_when_displaying_wrong_part_count() {
        let reason = InvalidUsageKeyReason::WrongPartCount { count: 2 };

        let expected_result = "the string has 2 parts, not 3";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }
}
