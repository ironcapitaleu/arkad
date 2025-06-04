//! # CIK Error Types
//!
//! This module defines error types and reasons for invalid SEC Central Index Keys (CIKs).
//! It is used throughout the [`crate::shared::cik`] module and by state machine implementations
//! that require robust error reporting for CIK parsing and validation failures.
//!
//! ## Types
//! - [`CikError`]: Error struct containing the [`InvalidCikReason`] and the offending CIK string. This allows precise diagnostics about why a CIK is invalid.
//! - [`InvalidCikReason`]: Enum describing specific reasons for CIK validation failure, such as exceeding the maximum length or containing non-numeric characters.
//!
//! ## Usage
//! These error types are returned by CIK parsing and validation routines, and are used in state data modules such as
//! [`crate::implementations::states::extract::validate_cik_format::vcf_data`] to provide detailed diagnostics and error handling.
//! They are also used as domain errors for the general state machine error logic in [`crate::error`] and are wrapped by state-level errors like [`crate::error::state_machine::state::InvalidCikFormat`].
//!
//! ## See Also
//! - [`crate::shared::cik`]: Main CIK utilities module.
//! - [`crate::error`]: Error types that may reference CIK errors for reporting.
//! - [`crate::error::state_machine::state::InvalidCikFormat`]: State-level error that wraps `CikError` for error propagation in state machines.

use thiserror::Error;

use super::constants::CIK_LENGTH;
/// Error details for an invalid CIK format.
///
/// This struct provides both the reason for the failure and the offending CIK string.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[CikError] Invalid CIK: Reason: '{reason}'. Input: '{invalid_cik}'.")]
pub struct CikError {
    /// The reason why the CIK is considered invalid.
    pub reason: InvalidCikReason,
    /// The invalid CIK string that was provided.
    pub invalid_cik: String,
}

impl CikError {
    /// Creates a new `CikError`.
    pub fn new(reason: InvalidCikReason, invalid_cik: impl Into<String>) -> Self {
        Self {
            reason,
            invalid_cik: invalid_cik.into(),
        }
    }
}

/// Enum representing the reason for an invalid CIK format.
///
/// This enum is marked as non-exhaustive to allow for future extension.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidCikReason {
    /// The CIK is too long.
    MaxLengthExceeded { cik_length: usize },
    /// The CIK contains non-numeric characters.
    ContainsNonNumericCharacters,
}

impl std::fmt::Display for InvalidCikReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxLengthExceeded { cik_length } => write!(
                f,
                "CIK cannot exceed {CIK_LENGTH} digits. Got: '{cik_length}'"
            ),
            Self::ContainsNonNumericCharacters => {
                write!(f, "CIK contains non-numeric chracters.")
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

        let expected_result = format!(
            "[CikError] Invalid CIK: Reason: '{}'. Input: '{}'.",
            reason, invalid_cik
        );

        let result = format!("{}", cik_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_reason_is_non_numeric() {
        let reason = InvalidCikReason::ContainsNonNumericCharacters;
        let invalid_cik = "12A4567890";
        let cik_error = CikError::new(reason.clone(), invalid_cik);

        let expected_result = format!(
            "[CikError] Invalid CIK: Reason: '{}'. Input: '{}'.",
            reason, invalid_cik
        );

        let result = format!("{}", cik_error);

        assert_eq!(result, expected_result);
    }
}
