//! # Invalid Window
//!
//! Provides [`InvalidWindow`], the error raised when a string is not a window.

use thiserror::Error;

/// Error raised when a string is not `day` or `minute`, so it cannot be a window.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidWindow] Window '{value}' is not 'day' or 'minute'")]
pub struct InvalidWindow {
    /// The string that was rejected.
    pub value: String,
}

impl InvalidWindow {
    /// Creates a new [`InvalidWindow`] for the rejected string.
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
    const fn should_implement_auto_traits_for_invalid_window() {
        implements_auto_traits::<InvalidWindow>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_window() {
        implements_value_traits::<InvalidWindow>();
    }

    #[test]
    fn should_name_the_rejected_string_when_displaying_invalid_window() {
        let error = InvalidWindow::new("hour");

        let expected_result = "[InvalidWindow] Window 'hour' is not 'day' or 'minute'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
