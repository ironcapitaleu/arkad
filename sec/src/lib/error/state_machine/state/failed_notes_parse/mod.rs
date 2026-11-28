//! # Failed Notes Parse Error
//!
//! Provides the [`FailedNotesParse`] error: a [`ParseErrorKind`] of the Notes parser, with the name
//! of the state in which the parse failed.

use std::error::Error;
use std::fmt;

use xbrl::error::parsing::ParseErrorKind;

use super::State as StateError;
use crate::traits::error::FromDomainError;

/// Error for a Notes filing that the parser rejects, tagged with the state it occurred in.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FailedNotesParse {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The error of the parser. It is boxed, because it is larger than the other state errors.
    pub parse_error: Box<ParseErrorKind>,
}

// Deviation: `Display` and `source()` are written by hand, because the derive of `thiserror`
// makes `source()` return the `Box` and not the `ParseErrorKind` in it.
impl fmt::Display for FailedNotesParse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[FailedNotesParse] Failed to parse the Notes filing, Caused by: {}",
            self.parse_error
        )
    }
}

impl Error for FailedNotesParse {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.parse_error.as_ref())
    }
}

impl FailedNotesParse {
    /// Creates a new [`FailedNotesParse`] error.
    #[must_use]
    pub fn new(state_name: impl Into<String>, parse_error: ParseErrorKind) -> Self {
        Self {
            state_name: state_name.into(),
            parse_error: Box::new(parse_error),
        }
    }
}

impl From<FailedNotesParse> for StateError {
    /// Wraps the error in the [`StateError::FailedNotesParse`] variant.
    fn from(error: FailedNotesParse) -> Self {
        Self::FailedNotesParse(error)
    }
}

impl FromDomainError<ParseErrorKind> for FailedNotesParse {
    type DomainErr = ParseErrorKind;

    /// Wraps a [`ParseErrorKind`] with the name of the state.
    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use xbrl::core::invalid_value::{InvalidValue, InvalidValueReason};

    use super::*;

    const STATE_NAME: &str = "Parse Notes Filing";

    fn invalid_submission_row() -> ParseErrorKind {
        ParseErrorKind::InvalidSubmissionRow {
            adsh: "0000320193-26-000001".to_owned(),
            source: InvalidValue::new("Form", InvalidValueReason::Empty, ""),
        }
    }

    #[test]
    fn should_chain_the_parse_error_after_caused_by_when_displayed() {
        let error = FailedNotesParse::new(STATE_NAME, invalid_submission_row());

        let expected_result = "[FailedNotesParse] Failed to parse the Notes filing, Caused by: [InvalidSubmissionRow] Invalid sub row of the Notes filing '0000320193-26-000001', Caused by: [InvalidValue] Invalid 'Form', Reason: 'The value is empty', Input: ''";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expose_the_parse_error_as_source_when_wrapping_it() {
        let error = FailedNotesParse::new(STATE_NAME, invalid_submission_row());

        let expected_result = Some(&invalid_submission_row());

        let result =
            Error::source(&error).and_then(|source| source.downcast_ref::<ParseErrorKind>());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_from_is_called() {
        let error = FailedNotesParse::new(STATE_NAME, invalid_submission_row());

        let expected_result = StateError::FailedNotesParse(FailedNotesParse {
            state_name: "Parse Notes Filing".to_owned(),
            parse_error: Box::new(invalid_submission_row()),
        });

        let result = StateError::from(error);

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_failed_notes_parse() {
        implements_auto_traits::<FailedNotesParse>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_failed_notes_parse() {
        implements_value_traits::<FailedNotesParse>();
    }
}
