//! # Missing Daily Index Error
//!
//! Provides the [`MissingDailyIndex`] error: the SEC has no daily index for a date, and no listing
//! shows an index for a later date. So the SEC has not published the index yet.

use thiserror::Error;

use super::State as StateError;
use crate::shared::index_date::IndexDate;

/// Error for a daily index that the SEC has not published yet, tagged with the state it occurred
/// in.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[MissingDailyIndex] Failure in State: '{state_name}', Reason: 'The SEC has not published the daily index of {index_date}'"
)]
pub struct MissingDailyIndex {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The date of the missing daily index.
    pub index_date: IndexDate,
}

impl MissingDailyIndex {
    /// Creates a new state-level [`MissingDailyIndex`] error.
    #[must_use]
    pub fn new(state_name: impl Into<String>, index_date: IndexDate) -> Self {
        Self {
            state_name: state_name.into(),
            index_date,
        }
    }
}

impl From<MissingDailyIndex> for StateError {
    /// Converts a [`MissingDailyIndex`] into a [`StateError::MissingDailyIndex`] variant.
    fn from(error: MissingDailyIndex) -> Self {
        Self::MissingDailyIndex(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn sample_error() -> MissingDailyIndex {
        let index_date = IndexDate::new("2026-10-27")
            .expect("Given a hardcoded past date, validation should always succeed");
        MissingDailyIndex::new("Download Daily Index", index_date)
    }

    #[test]
    fn should_convert_to_state_error_when_into_is_called() {
        let expected_result = StateError::MissingDailyIndex(sample_error());

        let result: StateError = sample_error().into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_date_when_displayed() {
        let expected_result = "[MissingDailyIndex] Failure in State: 'Download Daily Index', Reason: 'The SEC has not published the daily index of 2026-10-27'";

        let result = sample_error().to_string();

        assert_eq!(result, expected_result);
    }
}
