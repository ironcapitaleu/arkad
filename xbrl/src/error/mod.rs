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
//! ```

use thiserror::Error;

use self::parsing::ParseErrorKind;
use self::validation::ValidationErrorKind;

pub mod parsing;
pub mod validation;

/// Top-level error kind for the xbrl crate.
#[derive(Debug, Error)]
pub enum ErrorKind {
    /// An XBRL domain error occurred.
    #[error("[Xbrl] {0}")]
    Xbrl(#[source] XbrlErrorKind),
}

/// XBRL-specific error kind distinguishing parsing from validation failures.
#[derive(Debug, Error)]
pub enum XbrlErrorKind {
    /// Parsing of SEC JSON data failed.
    #[error("[FailedParsing] Caused by: {0}")]
    FailedParsing(#[source] ParseErrorKind),

    /// Validation of resolved financial data failed.
    #[error("[FailedValidation] Caused by: {0}")]
    FailedValidation(#[source] ValidationErrorKind),
}

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
