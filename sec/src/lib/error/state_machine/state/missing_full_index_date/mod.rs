//! # Missing Full Index Date Error
//!
//! Provides the [`MissingFullIndexDate`] error: the full index of a quarter does not cover a date
//! yet, because the date comes after the date of its header line `Last Data Received`.

use chrono::NaiveDate;
use thiserror::Error;

use super::State as StateError;

/// Error for a date that the full index of its quarter does not cover yet, tagged with the state
/// it occurred in.
///
/// The full index has no rows for such a date. Without this error, the date counts as a day
/// without filings, and no later read of the full index happens.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[MissingFullIndexDate] The full index does not cover the date yet, Date: '{date}', Last Data Received: '{last_data_received}'"
)]
pub struct MissingFullIndexDate {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The date that the full index does not cover.
    pub date: NaiveDate,
    /// The date of the header line `Last Data Received` of the full index.
    pub last_data_received: NaiveDate,
}

impl MissingFullIndexDate {
    /// Creates a new state-level [`MissingFullIndexDate`] error.
    #[must_use]
    pub fn new(
        state_name: impl Into<String>,
        date: NaiveDate,
        last_data_received: NaiveDate,
    ) -> Self {
        Self {
            state_name: state_name.into(),
            date,
            last_data_received,
        }
    }
}

impl From<MissingFullIndexDate> for StateError {
    /// Converts a [`MissingFullIndexDate`] into a [`StateError::MissingFullIndexDate`] variant.
    fn from(error: MissingFullIndexDate) -> Self {
        Self::MissingFullIndexDate(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn sample_error() -> MissingFullIndexDate {
        MissingFullIndexDate::new(
            "Select Full Index Filings",
            NaiveDate::from_ymd_opt(2026, 11, 13)
                .expect("Given a hardcoded calendar date, the date should always build"),
            NaiveDate::from_ymd_opt(2026, 11, 12)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    #[test]
    fn should_convert_to_state_error_when_into_is_called() {
        let missing_full_index_date = sample_error();

        let expected_result = StateError::MissingFullIndexDate(sample_error());

        let result: StateError = missing_full_index_date.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_date_and_the_last_data_received_when_displayed() {
        let missing_full_index_date = sample_error();

        let expected_result = "[MissingFullIndexDate] The full index does not cover the date yet, Date: '2026-11-13', Last Data Received: '2026-11-12'";

        let result = missing_full_index_date.to_string();

        assert_eq!(result, expected_result);
    }
}
