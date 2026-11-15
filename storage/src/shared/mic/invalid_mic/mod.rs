//! # Invalid MIC
//!
//! Provides [`InvalidMic`], the error raised when a string is not a MIC.

use thiserror::Error;

/// Error raised when a string is not four upper-case ASCII letters or digits.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidMic] MIC '{value}' is not four upper-case letters or digits")]
pub struct InvalidMic {
    /// The string that was rejected.
    pub value: String,
}

impl InvalidMic {
    /// Creates a new [`InvalidMic`] for the rejected string.
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
    const fn should_implement_auto_traits_for_invalid_mic() {
        implements_auto_traits::<InvalidMic>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_mic() {
        implements_value_traits::<InvalidMic>();
    }

    #[test]
    fn should_format_display_as_expected_for_an_invalid_mic() {
        let error = InvalidMic::new("xnas");

        let expected_result = "[InvalidMic] MIC 'xnas' is not four upper-case letters or digits";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
