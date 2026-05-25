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
    #[error("[Xbrl] Caused by: {0}")]
    Xbrl(#[source] XbrlErrorKind),

    /// Casting from [`ErrorKind`] to a more specific error type is not possible.
    #[error("[DowncastNotPossible] Cannot downcast to requested error type")]
    DowncastNotPossible,
}

/// XBRL-specific error kind distinguishing parsing from validation failures.
#[derive(Debug, PartialEq, Error)]
pub enum XbrlErrorKind {
    /// Parsing of SEC JSON data failed.
    #[error("[FailedParsing] Caused by: {0}")]
    FailedParsing(#[source] ParseErrorKind),

    /// Validation of resolved financial data failed.
    #[error("[FailedValidation] Caused by: {0}")]
    FailedValidation(#[source] ValidationErrorKind),
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
