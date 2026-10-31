//! # Daily Index Errors
//!
//! Provides the [`DailyIndexError`] returned by
//! [`parse_daily_index`](super::parse_daily_index), and the [`InvalidDailyIndexLineReason`]
//! describing why a line is malformed.

use std::fmt::{self, Display, Formatter};

use thiserror::Error;

use crate::shared::cik::CikError;

/// Error indicating that a daily index could not be parsed into rows.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DailyIndexError {
    /// The text has no line of dashes, so the parser cannot find where the rows start.
    #[error("[MissingDailyIndexHeader] The daily index has no line of dashes before its rows")]
    MissingDailyIndexHeader,
    /// A row line does not hold a form, a CIK, a date filed, and a file name.
    #[error(
        "[InvalidDailyIndexLine] Line {line_number} of the daily index is malformed, Reason: '{reason}'"
    )]
    InvalidDailyIndexLine {
        /// The number of the line in the whole text, counted from 1.
        line_number: usize,
        /// The reason why the line is malformed.
        reason: InvalidDailyIndexLineReason,
    },
    /// The CIK field of a row line fails [`Cik`](crate::shared::cik::Cik) validation.
    #[error(
        "[InvalidDailyIndexCik] Line {line_number} of the daily index has an invalid CIK, Caused by: {source}"
    )]
    InvalidDailyIndexCik {
        /// The number of the line in the whole text, counted from 1.
        line_number: usize,
        /// The error of the CIK validation.
        #[source]
        source: CikError,
    },
}

/// Enum representing the reason why a line of a daily index is malformed.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidDailyIndexLineReason {
    /// The line has fewer fields than a row.
    MissingField,
    /// The date filed field is not a date of the form `YYYYMMDD`.
    InvalidFiledDate {
        /// The text of the date filed field.
        filed_date: String,
    },
    /// The file name does not end in an accession number of the form `0000000000-00-000000.txt`.
    InvalidFileName {
        /// The text of the file name field.
        file_name: String,
    },
}

impl Display for InvalidDailyIndexLineReason {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::MissingField => write!(
                f,
                "Expected a form, a company name, a CIK, a date filed, and a file name"
            ),
            Self::InvalidFiledDate { filed_date } => write!(
                f,
                "The date filed '{filed_date}' is not a date of the form YYYYMMDD"
            ),
            Self::InvalidFileName { file_name } => write!(
                f,
                "The file name '{file_name}' does not end in an accession number and '.txt'"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::cik::InvalidCikReason;

    fn invalid_cik_error() -> DailyIndexError {
        DailyIndexError::InvalidDailyIndexCik {
            line_number: 12,
            source: CikError::new(InvalidCikReason::ContainsNonNumericCharacters, "12A"),
        }
    }

    #[test]
    fn should_format_display_when_header_is_missing() {
        let error = DailyIndexError::MissingDailyIndexHeader;

        let expected_result =
            "[MissingDailyIndexHeader] The daily index has no line of dashes before its rows";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_line_number_and_reason_when_field_is_missing() {
        let error = DailyIndexError::InvalidDailyIndexLine {
            line_number: 12,
            reason: InvalidDailyIndexLineReason::MissingField,
        };

        let expected_result = "[InvalidDailyIndexLine] Line 12 of the daily index is malformed, Reason: 'Expected a form, a company name, a CIK, a date filed, and a file name'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_cik_error_after_caused_by_when_cik_is_invalid() {
        let error = invalid_cik_error();

        let expected_result = format!(
            "[InvalidDailyIndexCik] Line 12 of the daily index has an invalid CIK, Caused by: {}",
            CikError::new(InvalidCikReason::ContainsNonNumericCharacters, "12A")
        );

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expose_cik_error_as_source_when_cik_is_invalid() {
        let error = invalid_cik_error();

        let expected_result =
            Some(CikError::new(InvalidCikReason::ContainsNonNumericCharacters, "12A").to_string());

        let result = error.source().map(ToString::to_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_filed_date_when_filed_date_is_invalid() {
        let reason = InvalidDailyIndexLineReason::InvalidFiledDate {
            filed_date: "20261332".to_string(),
        };

        let expected_result = "The date filed '20261332' is not a date of the form YYYYMMDD";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_file_name_when_file_name_is_invalid() {
        let reason = InvalidDailyIndexLineReason::InvalidFileName {
            file_name: "edgar/data/1/x.htm".to_string(),
        };

        let expected_result =
            "The file name 'edgar/data/1/x.htm' does not end in an accession number and '.txt'";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_daily_index_error() {
        implements_auto_traits::<DailyIndexError>();
    }
}
