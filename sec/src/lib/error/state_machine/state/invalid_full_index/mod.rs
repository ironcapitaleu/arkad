//! # Invalid Full Index Error
//!
//! Provides the [`InvalidFullIndex`] error: a domain-level [`FormIndexError`] with the name of
//! the state in which the parse of the full index failed.
//!
//! ## Example
//!
//! ```rust
//! use sec::error::state_machine::state::invalid_full_index::InvalidFullIndex;
//! use sec::shared::form_index_row::FormIndexError;
//!
//! let full_index_error = FormIndexError::MissingFormIndexHeader;
//! let state_error = InvalidFullIndex::new("Select Full Index Filings", full_index_error);
//!
//! let expected_result = "Select Full Index Filings";
//!
//! let result = state_error.state_name.as_str();
//!
//! assert_eq!(result, expected_result);
//! ```

use thiserror::Error;

use super::State as StateError;
use crate::shared::form_index_row::FormIndexError;
use crate::traits::error::FromDomainError;

/// Error for a full index that does not parse, tagged with the state it occurred in.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidFullIndex] Failed to parse the full index, Caused by: {full_index_error}")]
pub struct InvalidFullIndex {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level form index error.
    #[source]
    pub full_index_error: FormIndexError,
}

impl InvalidFullIndex {
    /// Creates a new state-level [`InvalidFullIndex`] error.
    #[must_use]
    pub fn new(state_name: impl Into<String>, domain_error: FormIndexError) -> Self {
        Self {
            state_name: state_name.into(),
            full_index_error: domain_error,
        }
    }
}

impl From<InvalidFullIndex> for StateError {
    /// Converts an [`InvalidFullIndex`] into a [`StateError::InvalidFullIndex`] variant.
    fn from(domain_error: InvalidFullIndex) -> Self {
        Self::InvalidFullIndex(domain_error)
    }
}

impl FromDomainError<FormIndexError> for InvalidFullIndex {
    type DomainErr = FormIndexError;

    /// Converts a domain-level [`FormIndexError`] into a state-level [`InvalidFullIndex`] error.
    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_convert_to_state_error_when_into_is_called() {
        let full_index_error = FormIndexError::MissingFormIndexHeader;
        let invalid_full_index = InvalidFullIndex::from_domain_error("TestState", full_index_error);

        let expected_result = StateError::InvalidFullIndex(invalid_full_index.clone());

        let result: StateError = invalid_full_index.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_form_index_error_as_source_of_invalid_full_index() {
        let full_index_error = FormIndexError::MissingFormIndexHeader;
        let invalid_full_index = InvalidFullIndex::new("TestState", full_index_error.clone());

        let expected_result = Some(&full_index_error);

        let result = std::error::Error::source(&invalid_full_index)
            .and_then(|source| source.downcast_ref::<FormIndexError>());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_the_error_chain_when_displayed() {
        let full_index_error = FormIndexError::MissingFormIndexHeader;
        let invalid_full_index =
            InvalidFullIndex::new("Select Full Index Filings", full_index_error);

        let expected_result = "[InvalidFullIndex] Failed to parse the full index, Caused by: [MissingFormIndexHeader] The index has no line that starts with '---' before its rows";

        let result = invalid_full_index.to_string();

        assert_eq!(result, expected_result);
    }
}
