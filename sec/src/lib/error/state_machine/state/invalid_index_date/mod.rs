//! # Invalid Index Date Error
//!
//! Provides the [`InvalidIndexDate`] error: a domain-level [`IndexDateError`] with the name of
//! the state in which the index date validation failed.
//!
//! ## Example
//!
//! ```rust
//! use sec::error::state_machine::state::invalid_index_date::InvalidIndexDate;
//! use sec::shared::index_date::{IndexDateError, InvalidIndexDateReason};
//!
//! let index_date_error =
//!     IndexDateError::new(InvalidIndexDateReason::InvalidIndexDateFormat, "2026-1-2");
//! let state_error = InvalidIndexDate::new("Validate Index Date", index_date_error);
//!
//! let expected_result = "Validate Index Date";
//!
//! let result = state_error.state_name.as_str();
//!
//! assert_eq!(result, expected_result);
//! ```

use thiserror::Error;

use super::State as StateError;
use crate::shared::index_date::IndexDateError;
use crate::traits::error::FromDomainError;

/// Error for an index date validation failure, tagged with the state it occurred in.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidIndexDate] Failure in State: '{state_name}', Caused by: {index_date_error}")]
pub struct InvalidIndexDate {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level index date error.
    #[source]
    pub index_date_error: IndexDateError,
}

impl InvalidIndexDate {
    /// Creates a new state-level [`InvalidIndexDate`] error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::error::state_machine::state::InvalidIndexDate;
    /// use sec::shared::index_date::{IndexDateError, InvalidIndexDateReason};
    ///
    /// let index_date_error =
    ///     IndexDateError::new(InvalidIndexDateReason::InvalidIndexDateFormat, "2026-13-01");
    /// let state_error = InvalidIndexDate::new("Validate Index Date", index_date_error.clone());
    ///
    /// let expected_result = index_date_error;
    ///
    /// let result = state_error.index_date_error;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn new(state_name: impl Into<String>, domain_error: IndexDateError) -> Self {
        Self {
            state_name: state_name.into(),
            index_date_error: domain_error,
        }
    }
}

impl From<InvalidIndexDate> for StateError {
    /// Converts an [`InvalidIndexDate`] into a [`StateError::InvalidIndexDate`] variant.
    fn from(domain_error: InvalidIndexDate) -> Self {
        Self::InvalidIndexDate(domain_error)
    }
}

impl FromDomainError<IndexDateError> for InvalidIndexDate {
    type DomainErr = IndexDateError;

    /// Converts a domain-level [`IndexDateError`] into a state-level [`InvalidIndexDate`] error.
    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::index_date::InvalidIndexDateReason;

    fn sample_index_date_error() -> IndexDateError {
        IndexDateError::new(InvalidIndexDateReason::InvalidIndexDateFormat, "2026-1-2")
    }

    #[test]
    fn should_convert_to_state_error_when_into_is_called() {
        let invalid_index_date =
            InvalidIndexDate::from_domain_error("TestState", sample_index_date_error());

        let expected_result = StateError::InvalidIndexDate(invalid_index_date.clone());

        let result: StateError = invalid_index_date.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_index_date_error_as_source_of_invalid_index_date() {
        let invalid_index_date = InvalidIndexDate::new("TestState", sample_index_date_error());

        let expected_result = Some(&sample_index_date_error());

        let result = std::error::Error::source(&invalid_index_date)
            .and_then(|source| source.downcast_ref::<IndexDateError>());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_the_error_chain_when_displayed() {
        let invalid_index_date =
            InvalidIndexDate::new("Validate Index Date", sample_index_date_error());

        let expected_result = "[InvalidIndexDate] Failure in State: 'Validate Index Date', Caused by: [IndexDateError] Invalid index date, Reason: 'Expected a calendar date of the form YYYY-MM-DD', Input: '2026-1-2'";

        let result = invalid_index_date.to_string();

        assert_eq!(result, expected_result);
    }
}
