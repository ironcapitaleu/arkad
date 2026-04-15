//! # Entity Name Module
//!
//! Provides the [`EntityName`] newtype for SEC entity (company) names.
//! Wraps the company name string reported in SEC filings
//! (e.g., `"Apple Inc."`, `"BERKSHIRE HATHAWAY INC"`).

use std::fmt;

/// An SEC entity (company) name.
///
/// Wraps the `entityName` field from SEC Company Facts API responses.
/// The name is stored exactly as reported by the SEC, without normalization.
///
/// # Example
/// ```
/// use sec::shared::financial::entity_name::EntityName;
///
/// let name = EntityName::new("Apple Inc.");
/// assert_eq!(name.value(), "Apple Inc.");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct EntityName(String);

impl EntityName {
    /// Creates a new [`EntityName`] from a string value.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns a reference to the entity name string.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EntityName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_value_when_entity_name_is_created() {
        let name = EntityName::new("Apple Inc.");

        let expected_result = "Apple Inc.";

        let result = name.value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_entity_name_when_formatted() {
        let name = EntityName::new("BERKSHIRE HATHAWAY INC");

        let expected_result = "BERKSHIRE HATHAWAY INC";

        let result = name.to_string();

        assert_eq!(result, expected_result);
    }
}
