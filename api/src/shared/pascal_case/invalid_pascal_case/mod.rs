//! # Invalid Pascal Case
//!
//! Provides the [`InvalidPascalCase`] error for a name that is not in `PascalCase`.

use thiserror::Error;

/// Error indicating that a name is not in `PascalCase`.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("[InvalidPascalCase] Name '{name}' is not in PascalCase")]
pub struct InvalidPascalCase {
    /// The name that failed the check.
    pub name: String,
}

impl InvalidPascalCase {
    /// Creates a new [`InvalidPascalCase`] error for `name`.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_invalid_pascal_case() {
        implements_auto_traits::<InvalidPascalCase>();
    }

    #[test]
    fn should_name_the_rejected_name_when_displaying_invalid_pascal_case() {
        let error = InvalidPascalCase::new("netIncome");

        let expected_result = "[InvalidPascalCase] Name 'netIncome' is not in PascalCase";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
