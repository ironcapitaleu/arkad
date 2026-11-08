//! # Invalid Value
//!
//! Provides [`InvalidValue`], the error that a core value type returns when its input breaks a rule.

use std::fmt;

use thiserror::Error;

/// The error that a constructor of a core value type returns when its input breaks a rule.
///
/// It is a leaf error. It names the type, the rule, and the input, and wraps no other error.
///
/// # Examples
///
/// ```
/// use xbrl::core::concept::NamespaceFamily;
/// use xbrl::core::invalid_value::{InvalidValue, InvalidValueReason};
///
/// let input = "";
///
/// let expected_result = Err(InvalidValue::new("NamespaceFamily", InvalidValueReason::Empty, input));
///
/// let result = NamespaceFamily::new(input);
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidValue] Invalid '{type_name}', Reason: '{reason}', Input: '{input}'")]
pub struct InvalidValue {
    /// The name of the type that rejected the input, for example `NamespaceFamily`.
    pub type_name: &'static str,
    /// The rule that the input breaks.
    pub reason: InvalidValueReason,
    /// The rejected input.
    pub input: String,
}

impl InvalidValue {
    /// Creates an [`InvalidValue`] for the type `type_name`, the broken rule, and the input.
    #[must_use]
    pub fn new(
        type_name: &'static str,
        reason: InvalidValueReason,
        input: impl Into<String>,
    ) -> Self {
        Self {
            type_name,
            reason,
            input: input.into(),
        }
    }
}

/// The rule that an input of a core value type breaks.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidValueReason {
    /// The input is empty.
    Empty,
    /// The input holds a whitespace character.
    ContainsWhitespace,
    /// The input holds a character that separates two parts of the value, for example `:`.
    ContainsSeparator(char),
    /// The input lacks the character that separates two parts of the value.
    MissingSeparator(char),
    /// The input names the same axis twice.
    DuplicateAxis,
    /// The end of a duration lies before its start.
    EndBeforeStart,
    /// The input is not a number.
    NotANumber,
    /// The input, or a number computed from it, lies outside the range of the type.
    OutOfRange,
    /// The input holds the placeholder, for example `{release}`, more or less than once.
    WrongPlaceholderCount(&'static str),
    /// The input lacks a required part, for example a URI pattern.
    MissingPart(&'static str),
    /// The input is not a hex digest of the expected length.
    NotHexDigest,
    /// The input is not a date in the format, for example `%Y%m%d`.
    NotADate(&'static str),
    /// No row of the table, for example `dim.tsv`, has the input as its key.
    NoMatchingRow(&'static str),
    /// The source cut the named field of the row that the input keys, so the field is not complete.
    Truncated(&'static str),
}

impl fmt::Display for InvalidValueReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "The value is empty"),
            Self::ContainsWhitespace => write!(f, "The value holds whitespace"),
            Self::ContainsSeparator(separator) => {
                write!(f, "The value holds the separator {separator}")
            }
            Self::MissingSeparator(separator) => {
                write!(f, "The value lacks the separator {separator}")
            }
            Self::DuplicateAxis => write!(f, "The value names one axis twice"),
            Self::EndBeforeStart => write!(f, "The end lies before the start"),
            Self::NotANumber => write!(f, "The value is not a number"),
            Self::OutOfRange => write!(f, "The value is out of range"),
            Self::WrongPlaceholderCount(placeholder) => {
                write!(f, "The value must hold {placeholder} exactly once")
            }
            Self::MissingPart(part) => write!(f, "The value lacks a {part}"),
            Self::NotHexDigest => write!(f, "The value is not a hex digest of the expected length"),
            Self::NotADate(format) => write!(f, "The value is not a date in the format {format}"),
            Self::NoMatchingRow(table) => write!(f, "No row of {table} has the value as its key"),
            Self::Truncated(field) => write!(f, "The source cut {field}"),
        }
    }
}

/// Returns the input if it is not empty and holds no whitespace.
///
/// # Errors
///
/// Returns [`InvalidValue`] with [`InvalidValueReason::Empty`] or
/// [`InvalidValueReason::ContainsWhitespace`].
pub(crate) fn require_token(type_name: &'static str, input: &str) -> Result<(), InvalidValue> {
    if input.is_empty() {
        return Err(InvalidValue::new(
            type_name,
            InvalidValueReason::Empty,
            input,
        ));
    }
    if input.chars().any(char::is_whitespace) {
        return Err(InvalidValue::new(
            type_name,
            InvalidValueReason::ContainsWhitespace,
            input,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_invalid_value() {
        implements_auto_traits::<InvalidValue>();
    }

    #[test]
    fn should_print_type_reason_and_input_when_displayed() {
        let error = InvalidValue::new(
            "Measure",
            InvalidValueReason::ContainsSeparator('/'),
            "USD/shares",
        );

        let expected_result = "[InvalidValue] Invalid 'Measure', Reason: 'The value holds the separator /', Input: 'USD/shares'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_input_when_it_holds_whitespace() {
        let input = "us gaap";

        let expected_result = Err(InvalidValue::new(
            "NamespaceFamily",
            InvalidValueReason::ContainsWhitespace,
            input,
        ));

        let result = require_token("NamespaceFamily", input);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_accept_input_when_it_is_a_token() {
        let input = "us-gaap";

        let expected_result = Ok(());

        let result = require_token("NamespaceFamily", input);

        assert_eq!(result, expected_result);
    }
}
