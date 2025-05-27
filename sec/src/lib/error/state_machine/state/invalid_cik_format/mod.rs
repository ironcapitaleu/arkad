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
    /// The reason why the CIK is considered invalid.
    pub reason: String,
    /// The invalid CIK string that was provided.
    pub invalid_cik: String,
}

impl std::fmt::Display for InvalidCikFormat {
    /// Formats the error for display, including the state name, reason, and offending CIK.
    ///
    /// # Example
    /// ```
    /// use sec::error::state_machine::state::invalid_cik_format::InvalidCikFormat;
    /// let err = InvalidCikFormat {
    ///     state_name: "SomeState".to_string(),
    ///     reason: "Too short".to_string(),
    ///     invalid_cik: "123".to_string(),
    /// };
    /// assert_eq!(
    ///     format!("{}", err),
    ///     "Invalid CIK: Too short. Input: '123'"
    /// );
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid CIK: {}. Input: '{}'",
            self.reason, self.invalid_cik
        )
    }
}

impl std::error::Error for InvalidCikFormat {}

impl InvalidCikFormat {
    /// Creates a new state-level [`InvalidCikFormat`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `reason` - The reason why the CIK is invalid.
    /// * `invalid_cik` - The offending CIK string.
    ///
    /// # Returns
    /// A new [`InvalidCikFormat`] error instance.
    #[must_use]
    pub fn new(
        state_name: &(impl ToString + ?Sized),
        reason: &(impl ToString + ?Sized),
        invalid_cik: &(impl ToString + ?Sized),
    ) -> Self {
        Self {
            state_name: state_name.to_string(),
            reason: reason.to_string(),
            invalid_cik: invalid_cik.to_string(),
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
impl FromDomainError for InvalidCikFormat {
    type DomainErr = CikError;

    /// Converts a domain-level [`CikError`] into a state-level [`InvalidCikFormat`] error.
    ///
    /// # Arguments
    /// * `err` - The domain-level [`CikError`] to wrap.
    /// * `state_name` - The name of the state where the error occurred.
    ///
    /// # Returns
    /// An [`InvalidCikFormat`] error containing the provided context.
    fn from_domain_error(err: Self::DomainErr, state_name: &(impl ToString + ?Sized)) -> Self {
        Self::new(state_name, &err.reason, &err.invalid_cik)
    }
}
