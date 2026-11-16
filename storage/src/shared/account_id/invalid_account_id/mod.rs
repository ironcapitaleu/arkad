//! # Invalid Account ID
//!
//! Provides [`InvalidAccountId`], the error raised when a string is not an account ID.

use thiserror::Error;

/// Error raised when a string is not a UUID, so it cannot be an account ID.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidAccountId] Account ID '{value}' is not a UUID")]
pub struct InvalidAccountId {
    /// The string that was rejected.
    pub value: String,
}

impl InvalidAccountId {
    /// Creates a new [`InvalidAccountId`] for the rejected string.
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
    const fn should_implement_auto_traits_for_invalid_account_id() {
        implements_auto_traits::<InvalidAccountId>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_account_id() {
        implements_value_traits::<InvalidAccountId>();
    }

    #[test]
    fn should_name_the_rejected_string_when_displaying_invalid_account_id() {
        let error = InvalidAccountId::new("not-a-uuid");

        let expected_result = "[InvalidAccountId] Account ID 'not-a-uuid' is not a UUID";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
