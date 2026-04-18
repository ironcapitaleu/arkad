//! # Accession Number Module
//!
//! Provides the [`AccessionNumber`] newtype for SEC filing accession numbers.
//! An accession number uniquely identifies a filing in the SEC EDGAR system
//! (e.g., `"0000320193-23-000106"`).

use std::fmt;

/// An SEC filing accession number.
///
/// A unique identifier assigned by the SEC to each filing submission.
/// Format is typically `{CIK}-{YY}-{sequence}` (e.g., `"0000320193-23-000106"`).
///
/// # Example
/// ```
/// use sec::shared::financial::accession_number::AccessionNumber;
///
/// let accn = AccessionNumber::new("0000320193-23-000106");
/// assert_eq!(accn.value(), "0000320193-23-000106");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct AccessionNumber(String);

impl AccessionNumber {
    /// Creates a new [`AccessionNumber`] from a string value.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns a reference to the accession number string.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AccessionNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_value_when_accession_number_is_created() {
        let accn = AccessionNumber::new("0000320193-23-000106");

        let expected_result = "0000320193-23-000106";

        let result = accn.value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_accession_number_when_formatted() {
        let accn = AccessionNumber::new("0000320193-23-000106");

        let expected_result = "0000320193-23-000106";

        let result = accn.to_string();

        assert_eq!(result, expected_result);
    }
}
