//! # Incomplete Company Facts Error
//!
//! Provides the [`IncompleteCompanyFacts`] error: an SEC Company Facts response that parsed as valid
//! JSON but is missing XBRL concepts required to build financial statements.
//!
//! ## Example
//!
//! ```rust
//! use sec::error::state_machine::state::incomplete_company_facts::{
//!     IncompleteCompanyFacts, MissingFields,
//! };
//!
//! let missing = MissingFields::new(vec!["Revenue".to_string(), "Total Assets".to_string()]);
//! let error = IncompleteCompanyFacts::new("Parse Company Facts", missing);
//! ```

use std::fmt;

use thiserror::Error;

use super::State as StateError;

/// The canonical names of the required concepts that are missing inside a response.
///
/// Formats as `["Revenue", "Total Assets"]`: brackets around the list, quotes around items,
/// comma-separated.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MissingFields(Vec<String>);

impl MissingFields {
    /// Creates a new [`MissingFields`] from a list of canonical field names.
    #[must_use]
    pub const fn new(fields: Vec<String>) -> Self {
        Self(fields)
    }

    /// Returns the underlying field names as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    /// Returns the number of missing fields.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if there are no missing fields.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for MissingFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = self
            .0
            .iter()
            .map(|s| format!("\"{s}\""))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "[{formatted}]")
    }
}

/// Error indicating an SEC Company Facts response is missing required concepts, tagged with the
/// state it occurred in.
///
    /// A *semantic* validation error, not a syntactic one: the JSON is valid, but some XBRL concepts
    /// that are required for financial statement construction were not found in the response. Carries
    /// the failing state's name and the list of missing concepts.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[IncompleteCompanyFacts] SEC response is missing expected data fields in State: '{state_name}', Reason: 'Missing fields: {missing_fields}'"
)]
pub struct IncompleteCompanyFacts {
    state_name: String,
    missing_fields: MissingFields,
}

impl IncompleteCompanyFacts {
    /// Creates a new [`IncompleteCompanyFacts`] error.
    #[must_use]
    pub fn new(state_name: impl Into<String>, missing_fields: MissingFields) -> Self {
        Self {
            state_name: state_name.into(),
            missing_fields,
        }
    }

    /// Returns the name of the state where the error occurred.
    #[must_use]
    pub fn state_name(&self) -> &str {
        &self.state_name
    }

    /// Returns the list of missing field canonical names.
    #[must_use]
    pub const fn missing_fields(&self) -> &MissingFields {
        &self.missing_fields
    }
}

impl From<IncompleteCompanyFacts> for StateError {
    /// Wraps the error in the [`StateError::IncompleteCompanyFacts`] variant.
    fn from(error: IncompleteCompanyFacts) -> Self {
        Self::IncompleteCompanyFacts(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_create_error_with_state_name_and_missing_fields_when_new_is_called() {
        let state_name = "Parse Company Facts";
        let missing = MissingFields::new(vec!["Revenue".to_string(), "Total Assets".to_string()]);

        let expected_result = IncompleteCompanyFacts {
            state_name: state_name.to_string(),
            missing_fields: missing.clone(),
        };

        let result = IncompleteCompanyFacts::new(state_name, missing);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_state_name_when_accessed() {
        let error = IncompleteCompanyFacts::new(
            "Parse Company Facts",
            MissingFields::new(vec!["Revenue".to_string()]),
        );

        let expected_result = "Parse Company Facts";

        let result = error.state_name();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_fields_when_accessed() {
        let missing = MissingFields::new(vec!["Revenue".to_string(), "Total Assets".to_string()]);
        let error = IncompleteCompanyFacts::new("TestState", missing.clone());

        let expected_result = &missing;

        let result = error.missing_fields();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_into_is_called() {
        let error = IncompleteCompanyFacts::new(
            "TestState",
            MissingFields::new(vec!["Revenue".to_string()]),
        );

        let expected_result = StateError::IncompleteCompanyFacts(error.clone());

        let result: StateError = error.into();

        assert_eq!(result, expected_result);
    }
}
