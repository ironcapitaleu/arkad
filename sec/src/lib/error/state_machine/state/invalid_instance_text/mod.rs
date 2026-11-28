//! # Invalid Instance Text Error
//!
//! Provides the [`InvalidInstanceText`] error: the bytes of a stored instance are not UTF-8 text.

use thiserror::Error;

use super::State as StateError;

/// Error for a stored instance whose bytes are not UTF-8, tagged with the state it occurred in.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[InvalidInstanceText] The instance of the filing '{filing}' is not UTF-8 text, Reason: '{reason}'"
)]
pub struct InvalidInstanceText {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The accession number of the filing.
    pub filing: String,
    /// The error of the UTF-8 decode.
    pub reason: String,
}

impl InvalidInstanceText {
    /// Creates a new [`InvalidInstanceText`] error.
    #[must_use]
    pub fn new(
        state_name: impl Into<String>,
        filing: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            state_name: state_name.into(),
            filing: filing.into(),
            reason: reason.into(),
        }
    }
}

impl From<InvalidInstanceText> for StateError {
    /// Wraps the error in the [`StateError::InvalidInstanceText`] variant.
    fn from(error: InvalidInstanceText) -> Self {
        Self::InvalidInstanceText(error)
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;

    const STATE_NAME: &str = "Parse Instance Filing";

    #[test]
    fn should_print_the_filing_and_the_reason_when_displayed() {
        let error = InvalidInstanceText::new(
            STATE_NAME,
            "0000320193-26-000001",
            "invalid utf-8 sequence of 1 bytes from index 3",
        );

        let expected_result = "[InvalidInstanceText] The instance of the filing '0000320193-26-000001' is not UTF-8 text, Reason: 'invalid utf-8 sequence of 1 bytes from index 3'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_from_is_called() {
        let error = InvalidInstanceText::new(STATE_NAME, "0000320193-26-000001", "bad byte");

        let expected_result = StateError::InvalidInstanceText(InvalidInstanceText {
            state_name: "Parse Instance Filing".to_owned(),
            filing: "0000320193-26-000001".to_owned(),
            reason: "bad byte".to_owned(),
        });

        let result = StateError::from(error);

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_invalid_instance_text() {
        implements_auto_traits::<InvalidInstanceText>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_invalid_instance_text() {
        implements_value_traits::<InvalidInstanceText>();
    }
}
