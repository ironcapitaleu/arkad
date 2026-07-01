//! # CIK Errors
//!
//! Provides the [`CikError`] returned when a string fails CIK validation, and the
//! [`InvalidCikReason`] describing why.
//!
//! These are domain errors raised by [`Cik::new`](super::Cik::new); state-level code wraps them
//! into [`InvalidCikFormat`](crate::error::state_machine::state::InvalidCikFormat) for propagation
//! through the state machine.

use thiserror::Error;

use super::constants::CIK_LENGTH;

/// Reports that a string could not be validated as a CIK.
///
/// Carries both the [`InvalidCikReason`] and the offending input, so callers can produce a
/// precise, user-facing diagnostic.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[CikError] Invalid CIK, Reason: '{reason}', Input: '{invalid_cik}'")]
pub struct CikError {
    /// Why the CIK is considered invalid.
    pub reason: InvalidCikReason,
    /// The original string that failed validation.
    pub invalid_cik: String,
}

impl CikError {
    /// Creates a new error from a reason and the offending input.
    pub fn new(reason: InvalidCikReason, invalid_cik: impl Into<String>) -> Self {
        Self {
            reason,
            invalid_cik: invalid_cik.into(),
        }
    }
}

/// Why a string failed CIK validation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidCikReason {
    /// The input, after trimming, exceeds [`CIK_LENGTH`] digits.
    MaxLengthExceeded {
        /// The actual digit count of the offending input.
        cik_length: usize,
    },
    /// The input contains characters that are not ASCII digits.
    ContainsNonNumericCharacters,
}

impl std::fmt::Display for InvalidCikReason {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::MaxLengthExceeded { cik_length } => write!(
                f,
                "CIK cannot exceed {CIK_LENGTH} digits. Got: '{cik_length}'"
            ),
            Self::ContainsNonNumericCharacters => {
                write!(f, "CIK contains non-numeric characters")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_format_display_as_expected_when_reason_is_too_long() {
        let invalid_cik = "123456789012345";
        let reason = InvalidCikReason::MaxLengthExceeded {
            cik_length: invalid_cik.len(),
        };
        let cik_error = CikError::new(reason.clone(), invalid_cik);

        let expected_result =
            format!("[CikError] Invalid CIK, Reason: '{reason}', Input: '{invalid_cik}'");

        let result = format!("{cik_error}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_reason_is_non_numeric() {
        let reason = InvalidCikReason::ContainsNonNumericCharacters;
        let invalid_cik = "12A4567890";
        let cik_error = CikError::new(reason.clone(), invalid_cik);

        let expected_result =
            format!("[CikError] Invalid CIK, Reason: '{reason}', Input: '{invalid_cik}'");

        let result = format!("{cik_error}");

        assert_eq!(result, expected_result);
    }
}
