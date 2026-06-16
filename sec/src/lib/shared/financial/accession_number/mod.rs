//! # Accession Number
//!
//! Provides the [`AccessionNumber`] newtype for SEC filing accession numbers.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

/// An SEC filing accession number — the unique ID of one EDGAR submission.
///
/// Stored verbatim in its typical `{CIK}-{YY}-{sequence}` shape (e.g. `"0000320193-23-000106"`);
/// the newtype keeps it distinct from other identifier strings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct AccessionNumber(String);

impl AccessionNumber {
    /// Creates an [`AccessionNumber`] from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::financial::accession_number::AccessionNumber;
    ///
    /// let accn = AccessionNumber::new("0000320193-23-000106");
    /// assert_eq!(accn.value(), "0000320193-23-000106");
    /// ```
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

impl Display for AccessionNumber {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
