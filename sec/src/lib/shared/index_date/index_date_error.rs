//! # Index Date Errors
//!
//! Provides the [`IndexDateError`] returned when a string fails index date validation, and the
//! [`InvalidIndexDateReason`] describing why.
//!
//! These are domain errors raised by [`IndexDate::new`](super::IndexDate::new).

use std::fmt::{self, Display, Formatter};

use chrono::NaiveDate;
use thiserror::Error;

/// Error indicating that a string could not be validated as an index date.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[IndexDateError] Invalid index date, Reason: '{reason}', Input: '{invalid_date}'")]
pub struct IndexDateError {
    /// The reason why the index date is invalid.
    pub reason: InvalidIndexDateReason,
    /// The original string that failed validation.
    pub invalid_date: String,
}

impl IndexDateError {
    /// Creates a new [`IndexDateError`].
    pub fn new(reason: InvalidIndexDateReason, invalid_date: impl Into<String>) -> Self {
        Self {
            reason,
            invalid_date: invalid_date.into(),
        }
    }
}

/// Enum representing the reason why a string failed index date validation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidIndexDateReason {
    /// The input is not a calendar date of the form `YYYY-MM-DD`.
    InvalidIndexDateFormat,
    /// The date is after today, so the SEC has no daily index for it.
    FutureIndexDate {
        /// The current UTC date at the time of validation.
        today: NaiveDate,
    },
}

impl Display for InvalidIndexDateReason {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InvalidIndexDateFormat => {
                write!(f, "Expected a calendar date of the form YYYY-MM-DD")
            }
            Self::FutureIndexDate { today } => {
                write!(f, "The date is after today, {}", today.format("%Y-%m-%d"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_format_display_with_reason_and_input_when_reason_is_invalid_format() {
        let error = IndexDateError::new(InvalidIndexDateReason::InvalidIndexDateFormat, "2026-1-2");

        let expected_result = "[IndexDateError] Invalid index date, Reason: 'Expected a calendar date of the form YYYY-MM-DD', Input: '2026-1-2'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_today_when_reason_is_future_index_date() {
        let today = NaiveDate::from_ymd_opt(2026, 1, 2)
            .expect("Given a hardcoded valid date, construction should always succeed");

        let expected_result = "The date is after today, 2026-01-02";

        let result = InvalidIndexDateReason::FutureIndexDate { today }.to_string();

        assert_eq!(result, expected_result);
    }
}
