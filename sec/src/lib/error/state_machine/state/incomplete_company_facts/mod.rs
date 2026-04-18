//! # Incomplete Company Facts State Error
//!
//! This module defines the [`IncompleteCompanyFacts`] error type, which indicates that
//! an SEC Company Facts response is semantically incomplete -- it is missing expected
//! data fields required for building financial statements.
//!
//! ## Purpose
//! - Signals that the SEC response is valid JSON but does not contain all the XBRL
//!   concepts required by the configured [`ConceptDefinition`](crate::shared::financial::concept_definition::ConceptDefinition)s.
//! - Provides a list of the specific fields that were expected but not found.
//!
//! ## Types
//! - [`IncompleteCompanyFacts`]: Struct containing the state name and list of missing fields.
//!
//! ## Example
//! ```rust
//! use sec::error::state_machine::state::incomplete_company_facts::IncompleteCompanyFacts;
//!
//! let error = IncompleteCompanyFacts::new(
//!     "Parse Company Facts",
//!     vec!["Revenue".to_string(), "Total Assets".to_string()],
//! );
//! assert_eq!(error.missing_fields().len(), 2);
//! ```

use thiserror::Error;

use super::State as StateError;

/// Error indicating that an SEC Company Facts response is missing expected data fields.
///
/// This is a semantic validation error, not a syntactic one: the JSON is valid,
/// but specific XBRL concepts that are required for financial statement construction
/// were not found in the response.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[IncompleteCompanyFacts] SEC response is missing expected data fields in State: '{state_name}', Reason: 'Missing fields: {missing_fields:?}'"
)]
pub struct IncompleteCompanyFacts {
    state_name: String,
    missing_fields: Vec<String>,
}

impl IncompleteCompanyFacts {
    /// Creates a new [`IncompleteCompanyFacts`] error.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where validation failed.
    /// * `missing_fields` - The canonical names of the concepts that were not found.
    #[must_use]
    pub fn new(state_name: impl Into<String>, missing_fields: Vec<String>) -> Self {
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
    pub fn missing_fields(&self) -> &[String] {
        &self.missing_fields
    }
}

impl From<IncompleteCompanyFacts> for StateError {
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
        let missing = vec!["Revenue".to_string(), "Total Assets".to_string()];

        let expected_result = IncompleteCompanyFacts {
            state_name: state_name.to_string(),
            missing_fields: missing.clone(),
        };

        let result = IncompleteCompanyFacts::new(state_name, missing);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_state_name_when_accessed() {
        let error = IncompleteCompanyFacts::new("Parse Company Facts", vec!["Revenue".to_string()]);

        let expected_result = "Parse Company Facts";

        let result = error.state_name();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_fields_when_accessed() {
        let missing = vec!["Revenue".to_string(), "Total Assets".to_string()];
        let error = IncompleteCompanyFacts::new("TestState", missing.clone());

        let expected_result = missing.as_slice();

        let result = error.missing_fields();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_into_is_called() {
        let error = IncompleteCompanyFacts::new("TestState", vec!["Revenue".to_string()]);

        let expected_result = StateError::IncompleteCompanyFacts(error.clone());

        let result: StateError = error.into();

        assert_eq!(result, expected_result);
    }
}
