/// Error details for an invalid CIK format.
///
/// This struct provides both the reason for the failure and the offending CIK string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InvalidCikFormatError {
    /// The reason why the CIK is considered invalid.
    pub reason: String,
    /// The invalid CIK string that was provided.
    pub invalid_cik: String,
}

impl std::fmt::Display for InvalidCikFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid CIK: {}. Input: '{}'",
            self.reason, self.invalid_cik
        )
    }
}

impl std::error::Error for InvalidCikFormatError {}

impl InvalidCikFormatError {
    /// Creates a new `InvalidCikFormatError`.
    pub fn new(reason: impl Into<String>, invalid_cik: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
            invalid_cik: invalid_cik.into(),
        }
    }
}