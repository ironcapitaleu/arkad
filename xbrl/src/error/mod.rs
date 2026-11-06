//! # Error Hierarchy
//!
//! Strongly-typed error types for XBRL parsing and validation failures.
//!
//! ## Structure
//!
//! ```text
//! ErrorKind
//!   ::Xbrl(XbrlErrorKind)
//!     ::FailedParsing(ParseErrorKind)
//!     ::FailedValidation(ValidationErrorKind)
//!   ::DowncastNotPossible
//! ```
//!
//! ## Conversions
//!
//! Specific errors upcast into [`ErrorKind`] via `From` / `.into()`.
//! Downcasting from [`ErrorKind`] to a specific variant uses `TryFrom`,
//! returning [`ErrorKind::DowncastNotPossible`] if the variant doesn't match.

use thiserror::Error;

use self::parsing::ParseErrorKind;
use self::validation::ValidationErrorKind;

pub mod parsing;
pub mod validation;

/// Top-level error kind for the xbrl crate.
#[derive(Debug, PartialEq, Error)]
pub enum ErrorKind {
    /// An XBRL domain error occurred.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    Xbrl(XbrlErrorKind),

    /// Casting from [`ErrorKind`] to a more specific error type is not possible.
    #[error("[DowncastNotPossible] Cannot downcast to requested error type")]
    DowncastNotPossible,
}

/// XBRL-specific error kind distinguishing parsing from validation failures.
#[derive(Debug, PartialEq, Error)]
pub enum XbrlErrorKind {
    /// Parsing of SEC JSON data failed.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    FailedParsing(ParseErrorKind),

    /// Validation of resolved financial data failed.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    FailedValidation(ValidationErrorKind),
}

// Upcasting (infallible) - converting from specific error types to the generic `ErrorKind``

impl From<ParseErrorKind> for ErrorKind {
    fn from(e: ParseErrorKind) -> Self {
        Self::Xbrl(XbrlErrorKind::FailedParsing(e))
    }
}

impl From<ValidationErrorKind> for ErrorKind {
    fn from(e: ValidationErrorKind) -> Self {
        Self::Xbrl(XbrlErrorKind::FailedValidation(e))
    }
}

impl From<XbrlErrorKind> for ErrorKind {
    fn from(e: XbrlErrorKind) -> Self {
        Self::Xbrl(e)
    }
}

// Downcasting (fallible) - trying to convert from a generic error to a specific one, returning an error if the variant cannot be downcast

impl TryFrom<ErrorKind> for ParseErrorKind {
    type Error = ErrorKind;

    fn try_from(e: ErrorKind) -> Result<Self, Self::Error> {
        match e {
            ErrorKind::Xbrl(XbrlErrorKind::FailedParsing(inner)) => Ok(inner),
            _ => Err(ErrorKind::DowncastNotPossible),
        }
    }
}

impl TryFrom<ErrorKind> for ValidationErrorKind {
    type Error = ErrorKind;

    fn try_from(e: ErrorKind) -> Result<Self, Self::Error> {
        match e {
            ErrorKind::Xbrl(XbrlErrorKind::FailedValidation(inner)) => Ok(inner),
            _ => Err(ErrorKind::DowncastNotPossible),
        }
    }
}

impl TryFrom<ErrorKind> for XbrlErrorKind {
    type Error = ErrorKind;

    fn try_from(e: ErrorKind) -> Result<Self, Self::Error> {
        match e {
            ErrorKind::Xbrl(inner) => Ok(inner),
            ErrorKind::DowncastNotPossible => Err(ErrorKind::DowncastNotPossible),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::error::validation::Invariant;

    #[test]
    fn should_print_only_the_leaf_when_a_validation_error_reaches_error_kind() {
        let validation_error = ValidationErrorKind::InconsistentIdentity {
            invariant: Invariant::BalanceSheetIdentity,
            left_value: 500_000_000,
            right_value: 490_000_000,
        };
        let error = ErrorKind::from(validation_error);

        let expected_result = "[InconsistentIdentity] Invariant 'BalanceSheetIdentity' violated, left=500000000, right=490000000";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_only_the_leaf_when_a_parse_error_reaches_error_kind() {
        let parse_error = ParseErrorKind::InvalidJson {
            reason: "unexpected EOF at position 4096".to_string(),
        };
        let error = ErrorKind::from(parse_error);

        let expected_result =
            "[InvalidJson] Failed to parse JSON body, Reason: 'unexpected EOF at position 4096'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_source_when_error_kind_wraps_a_leaf_parse_error() {
        let parse_error = ParseErrorKind::InvalidJson {
            reason: "unexpected EOF at position 4096".to_string(),
        };
        let error = ErrorKind::from(parse_error);

        let expected_result: Option<String> = None;

        let result = Error::source(&error).map(ToString::to_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_source_when_error_kind_wraps_a_leaf_validation_error() {
        let validation_error = ValidationErrorKind::InconsistentIdentity {
            invariant: Invariant::BalanceSheetIdentity,
            left_value: 500_000_000,
            right_value: 490_000_000,
        };
        let error = ErrorKind::from(validation_error);

        let expected_result: Option<String> = None;

        let result = Error::source(&error).map(ToString::to_string);

        assert_eq!(result, expected_result);
    }
}
