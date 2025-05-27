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
