//! State-level error type for invalid CIK format errors.
//!
//! This module defines [`InvalidCikFormat`], which enriches domain-level CIK errors with state context
//! for use in state machine error handling. It also provides conversions from domain errors and into
//! the [`StateError`] enum.

use super::State as StateError;
use crate::shared::cik::CikError;
use crate::traits::error::FromDomainError;

/// Error representing an invalid CIK format at the state level.
///
/// This error type is used to wrap domain-level [`CikError`]s with additional information about
/// the state in which the error occurred, making it suitable for use in state machine error handling.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InvalidCikFormat {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level CIK error.
    pub cik_error: CikError,
}

impl std::fmt::Display for InvalidCikFormat {
    /// Formats the error for display, including the state name, reason, and offending CIK.
    ///
    /// # Example
    /// ```
    /// use sec::shared::cik::{CikError, InvalidCikReason};
    /// use sec::error::state_machine::state::invalid_cik_format::InvalidCikFormat;
    /// let cik_error = CikError {
    ///     reason: InvalidCikReason::ContainsNonNumericCharacters,
    ///     invalid_cik: "abc".to_string(),
    /// };
    /// let err = InvalidCikFormat {
    ///     state_name: "SomeState".to_string(),
    ///     cik_error,
    /// };
    /// assert_eq!(
    ///     format!("{}", err),
    ///     format!("Invalid CIK: Reason: '{}'. Input: 'abc'.", InvalidCikReason::ContainsNonNumericCharacters)
    /// );
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid CIK: Reason: '{}'. Input: '{}'.",
            self.cik_error.reason, self.cik_error.invalid_cik
        )
    }
}

impl std::error::Error for InvalidCikFormat {}

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
    fn from(val: InvalidCikFormat) -> Self {
        Self::InvalidCikFormat(val)
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
}
