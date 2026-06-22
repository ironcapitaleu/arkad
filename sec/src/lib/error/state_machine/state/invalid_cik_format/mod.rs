//! # Invalid CIK Format Error
//!
//! Provides the [`InvalidCikFormat`] error: a domain-level [`CikError`] enriched with the name of
//! the state in which CIK validation failed.
//!
//! ## Example
//!
//! ```rust
//! use sec::error::state_machine::state::invalid_cik_format::InvalidCikFormat;
//! use sec::shared::cik::{CikError, InvalidCikReason};
//!
//! let cik_error = CikError::new(InvalidCikReason::ContainsNonNumericCharacters, "bad");
//! let state_error = InvalidCikFormat::new("Validate CIK Format", cik_error);
//! ```

use thiserror::Error;

use super::State as StateError;
use crate::shared::cik::CikError;
use crate::traits::error::FromDomainError;

/// A CIK validation failure, tagged with the state it occurred in.
///
/// Wraps a domain-level [`CikError`] together with the failing state's name, so a low-level
/// validation error carries state context as it propagates up the error hierarchy.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidCikFormat] Failure in State: '{state_name}', Caused by: {cik_error}")]
pub struct InvalidCikFormat {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level CIK error.
    #[source]
    pub cik_error: CikError,
}

impl InvalidCikFormat {
    /// Creates a new error from the failing state's name and the underlying CIK error.
    #[must_use]
    pub fn new(state_name: impl Into<String>, cik_error: CikError) -> Self {
        Self {
            state_name: state_name.into(),
            cik_error,
        }
    }
}

impl From<InvalidCikFormat> for StateError {
    fn from(domain_error: InvalidCikFormat) -> Self {
        Self::InvalidCikFormat(domain_error)
    }
}

impl FromDomainError<CikError> for InvalidCikFormat {
    type DomainErr = CikError;

    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name.into(), err)
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
            reason,
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
            cik_error,
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

        let expected_result = Some(&cik_error);

        let result = std::error::Error::source(&invalid_cik_format)
            .and_then(|source| source.downcast_ref::<CikError>());

        assert_eq!(result, expected_result);
    }
}
