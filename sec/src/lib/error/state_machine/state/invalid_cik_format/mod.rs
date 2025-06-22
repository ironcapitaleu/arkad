//! # Invalid CIK Format State Error
//!
//! This module defines the [`InvalidCikFormat`] error type, which represents CIK (Central Index Key) format errors
//! at the state level within the SEC state machine framework. It wraps domain-level [`CikError`]s with additional
//! state context, enabling precise error reporting and handling in state machine workflows.
//!
//! ## Purpose
//! - Enriches domain CIK validation errors with state information for robust error propagation.
//! - Supports conversion from domain errors and integration into the [`State`](super::State) error enum.
//!
//! ## Types
//! - [`InvalidCikFormat`]: Struct representing a CIK format error with state context.
//!
//! ## Usage
//! Use [`InvalidCikFormat`] to wrap [`CikError`]s when a CIK validation failure occurs within a state. This allows
//! downstream error handlers to access both the state context and the underlying domain error.
//!
//! ## Example
//! ```rust
//! use sec::error::state_machine::state::invalid_cik_format::InvalidCikFormat;
//! use sec::shared::cik::{CikError, InvalidCikReason};
//! let cik_error = CikError { reason: InvalidCikReason::ContainsNonNumericCharacters, invalid_cik: "bad".to_string() };
//! let state_error = InvalidCikFormat::new("ValidateCikFormat", cik_error);
//! ```
use thiserror::Error;

use super::State as StateError;
use crate::shared::cik::CikError;
use crate::traits::error::FromDomainError;

/// Error representing an invalid CIK format at the state level.
///
/// This error type is used to wrap domain-level [`CikError`]s with additional information about
/// the state in which the error occurred, making it suitable for use in state machine error handling.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidCikFormat] Failure in State: `{state_name}`. Invalid CIK: Reason: '{}'. Input: '{}'.", cik_error.reason, cik_error.invalid_cik)]
pub struct InvalidCikFormat {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level CIK error.
    #[source]
    pub cik_error: CikError,
}

impl InvalidCikFormat {
    /// Creates a new state-level [`InvalidCikFormat`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `cik_error` - The underlying domain-level CIK error.
    ///
    /// # Returns
    /// A new [`InvalidCikFormat`] error instance.
    #[must_use]
    pub fn new(state_name: &(impl ToString + ?Sized), cik_error: CikError) -> Self {
        Self {
            state_name: state_name.to_string(),
            cik_error,
        }
    }
}

/// Converts a state-level `InvalidCikFormat` error into the state error enum variant.
impl From<InvalidCikFormat> for StateError {
    /// Converts an [`InvalidCikFormat`] into a [`StateError::InvalidCikFormat`] variant.
    ///
    /// # Arguments
    /// * `val` - The [`InvalidCikFormat`] error to convert.
    ///
    /// # Returns
    /// A [`StateError`] containing the provided [`InvalidCikFormat`] error.
    fn from(domain_error: InvalidCikFormat) -> Self {
        Self::InvalidCikFormat(domain_error)
    }
}

/// Implements conversion from a domain-level [`CikError`] to a state-level [`InvalidCikFormat`] error.
///
/// This allows enriching a [`CikError`] with state context for use in state machine error handling.
impl FromDomainError<CikError> for InvalidCikFormat {
    type DomainErr = CikError;

    /// Converts a domain-level [`CikError`] into a state-level [`InvalidCikFormat`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `err` - The domain-level [`CikError`] to wrap.
    ///
    /// # Returns
    /// An [`InvalidCikFormat`] error containing the provided context.
    fn from_domain_error(state_name: &(impl ToString + ?Sized), err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}

#[cfg(test)]
mod tests {
    use crate::shared::cik::InvalidCikReason;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_create_invalid_cik_format_when_new_is_called() {
        let state_name = "TestState";
        let reason = InvalidCikReason::ContainsNonNumericCharacters;
        let invalid_cik = "12345";
        let cik_error = CikError {
            reason: reason,
            invalid_cik: invalid_cik.to_string(),
        };

        let expected_result = InvalidCikFormat {
            state_name: state_name.to_string(),
            cik_error: cik_error.clone(),
        };

        let result = InvalidCikFormat::new(state_name, cik_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_from_domain_error_when_from_domain_error_is_called() {
        let cik_error = CikError {
            reason: InvalidCikReason::ContainsNonNumericCharacters,
            invalid_cik: "abc".to_string(),
        };
        let state_name = "ParsingState";

        let expected_result = InvalidCikFormat {
            state_name: state_name.to_string(),
            cik_error: cik_error.clone(),
        };

        let result = InvalidCikFormat::from_domain_error(state_name, cik_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_into_is_called() {
        let cik_error = CikError {
            reason: InvalidCikReason::ContainsNonNumericCharacters,
            invalid_cik: "abc".to_string(),
        };
        let invalid_cik_format = InvalidCikFormat {
            state_name: "TestState".to_string(),
            cik_error: cik_error.clone(),
        };

        let expected_result = StateError::InvalidCikFormat(invalid_cik_format.clone());

        let result: StateError = invalid_cik_format.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_cik_error_as_source_of_invalid_cik_format() {
        let state_name = "SomeState";
        let cik_reason = InvalidCikReason::ContainsNonNumericCharacters;
        let invalid_cik = "12A45";
        let cik_error = CikError {
            reason: cik_reason,
            invalid_cik: invalid_cik.to_string(),
        };
        let invalid_cik_format = InvalidCikFormat::new(state_name, cik_error.clone());

        // Act
        let source = std::error::Error::source(&invalid_cik_format);

        // Assert
        // The source should be Some(&CikError)
        assert!(source.is_some(), "Expected source error to be present");
        let source = source.unwrap();

        let cik_error_from_source = source.downcast_ref::<CikError>();
        assert!(
            cik_error_from_source.is_some(),
            "Source should be CikError type"
        );
        assert_eq!(cik_error_from_source.unwrap(), &cik_error);
    }

    #[test]
    fn should_print_error_and_source_for_logging_demo() {
        // Arrange
        let state_name = "SomeState";
        let cik_reason = InvalidCikReason::ContainsNonNumericCharacters;
        let invalid_cik = "12A45";
        let cik_error = CikError {
            reason: cik_reason,
            invalid_cik: invalid_cik.to_string(),
        };
        let invalid_cik_format = InvalidCikFormat::new(state_name, cik_error.clone());

        // Act
        let error_string = format!("{}", invalid_cik_format);
        let source_string = match std::error::Error::source(&invalid_cik_format) {
            Some(source) => format!("{}", source),
            None => "No source error".to_string(),
        };

        // Print for demonstration (would be structured log in production)
        println!("Top-level error: {error_string}");
        println!("Caused by: {source_string}");

        // Assert (optional, for completeness)
        assert!(error_string.contains("Failure in State"));
        assert!(source_string.contains("Invalid CIK"));
    }
}
