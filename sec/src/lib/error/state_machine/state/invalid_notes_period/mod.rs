//! # Invalid Notes Period Error
//!
//! Provides the [`InvalidNotesPeriod`] error: a domain-level [`NotesPeriodError`] with the name of
//! the state in which the Notes period validation failed.
//!
//! ## Example
//!
//! ```rust
//! use sec::error::state_machine::state::invalid_notes_period::InvalidNotesPeriod;
//! use sec::shared::notes_period::{InvalidNotesPeriodReason, NotesPeriodError};
//!
//! let notes_period_error =
//!     NotesPeriodError::new(InvalidNotesPeriodReason::InvalidNotesPeriodFormat, "2024");
//! let state_error = InvalidNotesPeriod::new("Validate Notes Period", notes_period_error);
//!
//! let expected_result = "Validate Notes Period";
//!
//! let result = state_error.state_name.as_str();
//!
//! assert_eq!(result, expected_result);
//! ```

use thiserror::Error;

use super::State as StateError;
use crate::shared::notes_period::NotesPeriodError;
use crate::traits::error::FromDomainError;

/// Error representing a Notes period validation failure, tagged with the state it occurred in.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[InvalidNotesPeriod] Failure in State: '{state_name}', Caused by: {notes_period_error}")]
pub struct InvalidNotesPeriod {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The underlying domain-level Notes period error.
    #[source]
    pub notes_period_error: NotesPeriodError,
}

impl InvalidNotesPeriod {
    /// Creates a new state-level [`InvalidNotesPeriod`] error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::error::state_machine::state::InvalidNotesPeriod;
    /// use sec::shared::notes_period::{InvalidNotesPeriodReason, NotesPeriodError};
    ///
    /// let notes_period_error =
    ///     NotesPeriodError::new(InvalidNotesPeriodReason::UnpublishedQuarter, "2025q4");
    /// let state_error = InvalidNotesPeriod::new("Validate Notes Period", notes_period_error.clone());
    ///
    /// let expected_result = notes_period_error;
    ///
    /// let result = state_error.notes_period_error;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn new(state_name: impl Into<String>, domain_error: NotesPeriodError) -> Self {
        Self {
            state_name: state_name.into(),
            notes_period_error: domain_error,
        }
    }
}

impl From<InvalidNotesPeriod> for StateError {
    /// Converts an [`InvalidNotesPeriod`] into a [`StateError::InvalidNotesPeriod`] variant.
    fn from(domain_error: InvalidNotesPeriod) -> Self {
        Self::InvalidNotesPeriod(domain_error)
    }
}

impl FromDomainError<NotesPeriodError> for InvalidNotesPeriod {
    type DomainErr = NotesPeriodError;

    /// Converts a domain-level [`NotesPeriodError`] into a state-level [`InvalidNotesPeriod`] error.
    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::notes_period::InvalidNotesPeriodReason;

    fn sample_notes_period_error() -> NotesPeriodError {
        NotesPeriodError::new(InvalidNotesPeriodReason::InvalidNotesPeriodFormat, "2024")
    }

    #[test]
    fn should_convert_to_state_error_when_into_is_called() {
        let invalid_notes_period =
            InvalidNotesPeriod::from_domain_error("TestState", sample_notes_period_error());

        let expected_result = StateError::InvalidNotesPeriod(invalid_notes_period.clone());

        let result: StateError = invalid_notes_period.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_notes_period_error_as_source_of_invalid_notes_period() {
        let invalid_notes_period =
            InvalidNotesPeriod::new("TestState", sample_notes_period_error());

        let expected_result = Some(&sample_notes_period_error());

        let result = std::error::Error::source(&invalid_notes_period)
            .and_then(|source| source.downcast_ref::<NotesPeriodError>());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_the_error_chain_when_displayed() {
        let invalid_notes_period =
            InvalidNotesPeriod::new("Validate Notes Period", sample_notes_period_error());

        let expected_result = "[InvalidNotesPeriod] Failure in State: 'Validate Notes Period', Caused by: [NotesPeriodError] Invalid Notes period, Reason: 'Expected the form YYYYqN or YYYY_MM', Input: '2024'";

        let result = invalid_notes_period.to_string();

        assert_eq!(result, expected_result);
    }
}
