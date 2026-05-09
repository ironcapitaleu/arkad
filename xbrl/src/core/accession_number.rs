//! # Accession Number
//!
//! SEC filing unique identifier wrapper.

use std::fmt;

/// A unique identifier for an SEC filing (e.g., `"0000320193-24-000123"`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct AccessionNumber(String);

impl AccessionNumber {
    /// Creates a new [`AccessionNumber`] from a raw string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the underlying string value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AccessionNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
