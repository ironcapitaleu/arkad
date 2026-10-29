//! # Notes Period Errors
//!
//! Provides the [`NotesPeriodError`] returned when a string fails Notes period validation, and the
//! [`InvalidNotesPeriodReason`] describing why.
//!
//! These are domain errors raised by [`NotesPeriod::new`](super::NotesPeriod::new).

use std::fmt::{self, Display, Formatter};

use thiserror::Error;

use super::constants::{FIRST_MONTH, FIRST_YEAR, LAST_QUARTER};
use super::{NotesPeriod, PeriodPart};

/// Error indicating that a string could not be validated as a Notes period.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[NotesPeriodError] Invalid Notes period, Reason: '{reason}', Input: '{invalid_period}'")]
pub struct NotesPeriodError {
    /// The reason why the Notes period is invalid.
    pub reason: InvalidNotesPeriodReason,
    /// The original string that failed validation.
    pub invalid_period: String,
}

impl NotesPeriodError {
    /// Creates a new [`NotesPeriodError`].
    pub fn new(reason: InvalidNotesPeriodReason, invalid_period: impl Into<String>) -> Self {
        Self {
            reason,
            invalid_period: invalid_period.into(),
        }
    }
}

/// Enum representing the reason why a string failed Notes period validation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidNotesPeriodReason {
    /// The input matches neither `YYYYqN` nor `YYYY_MM`.
    InvalidNotesPeriodFormat,
    /// The year comes before [`FIRST_YEAR`].
    UnpublishedYear {
        /// The year of the input.
        year: u16,
    },
    /// The quarter comes after [`LAST_QUARTER`].
    UnpublishedQuarter,
    /// The month comes before [`FIRST_MONTH`].
    UnpublishedMonth,
}

impl Display for InvalidNotesPeriodReason {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InvalidNotesPeriodFormat => {
                write!(f, "Expected the form 'YYYYqN' or 'YYYY_MM'")
            }
            Self::UnpublishedYear { year } => {
                write!(f, "The first Notes year is {FIRST_YEAR}, got {year}")
            }
            Self::UnpublishedQuarter => {
                let (year, quarter) = LAST_QUARTER;
                let last_quarter = NotesPeriod {
                    year,
                    part: PeriodPart::Quarter(quarter),
                };
                write!(f, "The last quarterly Notes file is {last_quarter}")
            }
            Self::UnpublishedMonth => {
                let (year, month) = FIRST_MONTH;
                let first_month = NotesPeriod {
                    year,
                    part: PeriodPart::Month(month),
                };
                write!(f, "The first monthly Notes file is {first_month}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_format_display_with_reason_and_input_when_reason_is_unpublished_year() {
        let error = NotesPeriodError::new(
            InvalidNotesPeriodReason::UnpublishedYear { year: 2008 },
            "2008q4",
        );

        let expected_result = "[NotesPeriodError] Invalid Notes period, Reason: 'The first Notes year is 2009, got 2008', Input: '2008q4'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_both_forms_when_reason_is_invalid_format() {
        let expected_result = "Expected the form 'YYYYqN' or 'YYYY_MM'";

        let result = InvalidNotesPeriodReason::InvalidNotesPeriodFormat.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_last_quarter_when_reason_is_unpublished_quarter() {
        let expected_result = "The last quarterly Notes file is 2025q3";

        let result = InvalidNotesPeriodReason::UnpublishedQuarter.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_first_month_when_reason_is_unpublished_month() {
        let expected_result = "The first monthly Notes file is 2025_10";

        let result = InvalidNotesPeriodReason::UnpublishedMonth.to_string();

        assert_eq!(result, expected_result);
    }
}
