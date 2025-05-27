use super::constants::CIK_LENGTH;
/// Error details for an invalid CIK format.
///
/// This struct provides both the reason for the failure and the offending CIK string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CikError {
    /// The reason why the CIK is considered invalid.
    pub reason: InvalidCikReason,
    /// The invalid CIK string that was provided.
    pub invalid_cik: String,
}

impl std::fmt::Display for CikError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid CIK: Reason: '{}'. Input: '{}'.",
            self.reason, self.invalid_cik
        )
    }
}

impl std::error::Error for CikError {}

impl CikError {
    /// Creates a new `CikError`.
    pub fn new(reason: InvalidCikReason, invalid_cik: impl Into<String>) -> Self {
        Self {
            reason: reason,
            invalid_cik: invalid_cik.into(),
        }
    }
}

/// Reason for an invalid CIK format.
///
/// This enum is marked as non-exhaustive to allow for future extension.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidCikReason {
    /// The CIK is too long.
    TooLong { cik_length: usize },
    /// The CIK contains non-numeric characters.
    NonNumeric,
}

impl std::fmt::Display for InvalidCikReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidCikReason::TooLong { cik_length } => write!(
                f,
                "CIK cannot exceed {CIK_LENGTH} digits. Got: '{cik_length}'"
            ),
            InvalidCikReason::NonNumeric => write!(f, "Non-numeric"),
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
        let reason = InvalidCikReason::TooLong {
            cik_length: invalid_cik.len(),
        };
        let cik_error = CikError::new(reason.clone(), invalid_cik);

        let expected_result = format!("Invalid CIK: Reason: '{}'. Input: '{}'.", reason, invalid_cik);

        let result = format!("{}", cik_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_reason_is_non_numeric() {
        let reason = InvalidCikReason::NonNumeric;
        let invalid_cik = "12A4567890";
        let cik_error = CikError::new(reason.clone(), invalid_cik);

        let expected_result = format!("Invalid CIK: Reason: '{}'. Input: '{}'.", reason, invalid_cik);

        let result = format!("{}", cik_error);

        assert_eq!(result, expected_result);
    }
}
