//! # Invalid Job Kind
//!
//! Provides [`InvalidJobKind`], the error raised when a string is not a job kind.

use thiserror::Error;

/// Error raised when a string is empty, or holds a character other than lower-case ASCII letters,
/// digits, and `-`.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[InvalidJobKind] Job kind '{value}' is not one or more lower-case ASCII letters, digits, or '-'"
)]
pub struct InvalidJobKind {
    /// The string that was rejected.
    pub value: String,
}

impl InvalidJobKind {
    /// Creates a new [`InvalidJobKind`] for the rejected string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_name_the_rejected_value_when_displaying_the_error() {
        let error = InvalidJobKind::new("Series");

        let expected_result = "[InvalidJobKind] Job kind 'Series' is not one or more lower-case ASCII letters, digits, or '-'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
