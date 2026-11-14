//! # Invalid Filed Date Range Error
//!
//! Provides the [`InvalidFiledDateRange`] error: the range of dates filed of a full index
//! selection is not inside one quarter, or its first date comes after its last.

use chrono::NaiveDate;
use thiserror::Error;

use super::State as StateError;

/// Error for a range of dates filed that one full index cannot cover, tagged with the state it
/// occurred in.
///
/// A full index lists one quarter. A range with its first date after its last, or with its dates
/// in two quarters, selects no filing or misses filings, and the run reports success.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[InvalidFiledDateRange] The range of dates filed must be in order and inside one quarter, First: '{first_filed_date}', Last: '{last_filed_date}'"
)]
pub struct InvalidFiledDateRange {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The first date filed of the range.
    pub first_filed_date: NaiveDate,
    /// The last date filed of the range.
    pub last_filed_date: NaiveDate,
}

impl InvalidFiledDateRange {
    /// Creates a new state-level [`InvalidFiledDateRange`] error.
    #[must_use]
    pub fn new(
        state_name: impl Into<String>,
        first_filed_date: NaiveDate,
        last_filed_date: NaiveDate,
    ) -> Self {
        Self {
            state_name: state_name.into(),
            first_filed_date,
            last_filed_date,
        }
    }
}

impl From<InvalidFiledDateRange> for StateError {
    /// Converts an [`InvalidFiledDateRange`] into a [`StateError::InvalidFiledDateRange`] variant.
    fn from(error: InvalidFiledDateRange) -> Self {
        Self::InvalidFiledDateRange(error)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn sample_error() -> InvalidFiledDateRange {
        InvalidFiledDateRange::new(
            "Select Full Index Filings",
            NaiveDate::from_ymd_opt(2026, 9, 30)
                .expect("Given a hardcoded calendar date, the date should always build"),
            NaiveDate::from_ymd_opt(2026, 9, 29)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    #[test]
    fn should_convert_to_state_error_when_into_is_called() {
        let invalid_filed_date_range = sample_error();

        let expected_result = StateError::InvalidFiledDateRange(sample_error());

        let result: StateError = invalid_filed_date_range.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_both_dates_when_displayed() {
        let invalid_filed_date_range = sample_error();

        let expected_result = "[InvalidFiledDateRange] The range of dates filed must be in order and inside one quarter, First: '2026-09-30', Last: '2026-09-29'";

        let result = invalid_filed_date_range.to_string();

        assert_eq!(result, expected_result);
    }
}
