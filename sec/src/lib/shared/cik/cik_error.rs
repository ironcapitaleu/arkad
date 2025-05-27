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
            "Invalid CIK: {}. Input: '{}'",
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
    TooLong,
    /// The CIK contains non-numeric characters.
    NonNumeric,
    /// The CIK is empty.
    Empty,
}

impl std::fmt::Display for InvalidCikReason {
    /// Formats the reason for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidCikReason::TooLong => write!(f, "Too long"),
            InvalidCikReason::NonNumeric => write!(f, "Non-numeric"),
            InvalidCikReason::Empty => write!(f, "Empty"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_format_display_as_expected_when_reason_is_too_long() {
        let reason = InvalidCikReason::TooLong;
        let invalid_cik = "123456789012345";
        let cik_error = CikError::new(reason, invalid_cik);

        let expected_result = "Invalid CIK: Too long. Input: '123456789012345'".to_string();

        let result = format!("{}", cik_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_reason_is_non_numeric() {
        // Arrange
        let reason = InvalidCikReason::NonNumeric;
        let invalid_cik = "12A4567890";
        let cik_error = CikError::new(reason, invalid_cik);

        // Define
        let expected_result = "Invalid CIK: Non-numeric. Input: '12A4567890'".to_string();

        // Act
        let result = format!("{}", cik_error);

        // Assert
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_reason_is_empty() {
        // Arrange
        let reason = InvalidCikReason::Empty;
        let invalid_cik = "";
        let cik_error = CikError::new(reason, invalid_cik);

        // Define
        let expected_result = "Invalid CIK: Empty. Input: ''".to_string();

        // Act
        let result = format!("{}", cik_error);

        // Assert
        assert_eq!(result, expected_result);
    }
}
