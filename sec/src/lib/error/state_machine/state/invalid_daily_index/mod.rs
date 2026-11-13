//! # Invalid Daily Index Error
//!
//! Provides the [`InvalidDailyIndex`] error: a domain-level [`FormIndexError`] with the name of
//! the state in which the parse of the daily index failed.
//!
//! ## Example
//!
//! ```rust
//! use sec::error::state_machine::state::invalid_daily_index::InvalidDailyIndex;
//! use sec::shared::form_index_row::FormIndexError;
//!
//! let daily_index_error = FormIndexError::MissingFormIndexHeader;
//! let state_error = InvalidDailyIndex::new("Select Filings", daily_index_error);
//!
//! let expected_result = "Select Filings";
//!
//! let result = state_error.state_name.as_str();
//!
//! assert_eq!(result, expected_result);
//! ```

use thiserror::Error;

use super::State as StateError;
use crate::shared::form_index_row::FormIndexError;
use crate::traits::error::FromDomainError;

/// Error for a daily index that does not parse, tagged with the state it occurred in.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidDailyIndex] Failed to parse the daily index, Caused by: {daily_index_error}")]
pub struct InvalidDailyIndex {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level daily index error.
    #[source]
    pub daily_index_error: FormIndexError,
}

impl InvalidDailyIndex {
    /// Creates a new state-level [`InvalidDailyIndex`] error.
    #[must_use]
    pub fn new(state_name: impl Into<String>, domain_error: FormIndexError) -> Self {
        Self {
            state_name: state_name.into(),
            daily_index_error: domain_error,
        }
    }
}

impl From<InvalidDailyIndex> for StateError {
    /// Converts an [`InvalidDailyIndex`] into a [`StateError::InvalidDailyIndex`] variant.
    fn from(domain_error: InvalidDailyIndex) -> Self {
        Self::InvalidDailyIndex(domain_error)
    }
}

impl FromDomainError<FormIndexError> for InvalidDailyIndex {
    type DomainErr = FormIndexError;

    /// Converts a domain-level [`FormIndexError`] into a state-level [`InvalidDailyIndex`] error.
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
        let daily_index_error = FormIndexError::MissingFormIndexHeader;
        let invalid_daily_index =
            InvalidDailyIndex::from_domain_error("TestState", daily_index_error);

        let expected_result = StateError::InvalidDailyIndex(invalid_daily_index.clone());

        let result: StateError = invalid_daily_index.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_daily_index_error_as_source_of_invalid_daily_index() {
        let daily_index_error = FormIndexError::MissingFormIndexHeader;
        let invalid_daily_index = InvalidDailyIndex::new("TestState", daily_index_error.clone());

        let expected_result = Some(&daily_index_error);

        let result = std::error::Error::source(&invalid_daily_index)
            .and_then(|source| source.downcast_ref::<FormIndexError>());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_the_error_chain_when_displayed() {
        let daily_index_error = FormIndexError::MissingFormIndexHeader;
        let invalid_daily_index = InvalidDailyIndex::new("Select Filings", daily_index_error);

        let expected_result = "[InvalidDailyIndex] Failed to parse the daily index, Caused by: [MissingFormIndexHeader] The index has no line that starts with '---' before its rows";

        let result = invalid_daily_index.to_string();

        assert_eq!(result, expected_result);
    }
}
