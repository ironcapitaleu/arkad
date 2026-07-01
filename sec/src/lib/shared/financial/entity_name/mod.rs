//! # Entity Name
//!
//! Provides the [`EntityName`] newtype for SEC entity (company) names.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

/// An SEC entity (company) name.
///
/// Wraps the `entityName` field from a Company Facts response, stored verbatim without
/// normalization, so a company name can't be mixed up with other strings in the domain.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct EntityName(String);

impl EntityName {
    /// Creates an [`EntityName`] from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::financial::entity_name::EntityName;
    ///
    /// let name = EntityName::new("Apple Inc.");
    /// assert_eq!(name.value(), "Apple Inc.");
    /// ```
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

impl Display for EntityName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
