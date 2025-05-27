//! State-level error type for invalid CIK format errors.
//!
//! This module defines [`InvalidCikFormat`], which enriches domain-level CIK errors with state context
//! for use in state machine error handling. It also provides conversions from domain errors and into
//! the state error enum.

use super::State as StateError;
use crate::implementations::states::extract::validate_cik_format::error::CikError;

/// Error details for an invalid CIK format at the state level.
///
/// This struct provides the reason for the failure, the offending CIK string, and the state name
/// where the error occurred.
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
    /// Creates a new state-level `InvalidCikFormat` error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `reason` - The reason why the CIK is invalid.
    /// * `invalid_cik` - The offending CIK string.
    #[must_use]
    pub fn new(
        state_name: impl Into<String>,
        reason: impl Into<String>,
        invalid_cik: impl Into<String>,
    ) -> Self {
        Self {
            state_name: state_name.into(),
            reason: reason.into(),
            invalid_cik: invalid_cik.into(),
        }
    }
}

/// Converts a domain-level `CikError` and a state name into a state-level `InvalidCikFormat` error.
impl From<(CikError, String)> for InvalidCikFormat {
    fn from((domain_error, state_name): (CikError, String)) -> Self {
        Self {
            reason: domain_error.reason,
            invalid_cik: domain_error.invalid_cik,
            state_name,
        }
    }
}

/// Converts a state-level `InvalidCikFormat` error into the state error enum variant.
impl Into<StateError> for InvalidCikFormat {
    fn into(self) -> StateError {
        StateError::InvalidCikFormat(self)
    }
}
