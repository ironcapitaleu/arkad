//! # Run Event
//!
//! Provides [`RunEvent`], [`ErrorKind`], and [`Unit`], the values that a batch run passes to its
//! observer.
//!
//! A batch run calls the observer once for each stream item and once for each error, at the
//! moment the machine emits it. So the caller can log during the run, not only after it.

use std::fmt::{self, Display, Formatter};

use chrono::NaiveDate;
use serde::{Serialize, Serializer};

use crate::prelude::{StreamError, StreamItem};
use crate::shared::cik::Cik;
use crate::shared::financial::accession_number::AccessionNumber;

/// One event of a batch run: a successful stream item, or an error and its kind.
#[derive(Debug, Clone, Copy)]
pub enum RunEvent<'a> {
    /// A successful stream item of the machine of the unit.
    Item(&'a StreamItem),
    /// The error of the machine of the unit, and how the run counts it.
    Error(&'a StreamError, ErrorKind),
}

/// How a batch run counts an error.
///
/// The caller picks the log level from the kind. The summary of the run, not the kind, decides
/// the exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorKind {
    /// The SEC answered 503, and the machine runs again.
    Retried,
    /// The daily index does not exist yet, and a later run reads the date again.
    Pending,
    /// The SEC answered 403 or 404 for the file. The summary decides whether the unit is
    /// unpublished or failed.
    Absent,
    /// The folder of the filing holds no XBRL instance.
    Unextractable,
    /// The error fails the run.
    Failed,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retried => write!(f, "retried"),
            Self::Pending => write!(f, "pending"),
            Self::Absent => write!(f, "absent"),
            Self::Unextractable => write!(f, "unextractable"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// The unit of a batch run that an event belongs to.
///
/// [`Display`] gives the values without keys, for example `2024q1` or `2026-10-26 CIK 0000003545`.
/// [`Serialize`] gives an object with one key per value, for example `{"period": "2024q1"}` or
/// `{"date": "2026-10-26", "cik": "0000003545"}`. The key `cik` of [`Unit::Cik`] holds the raw
/// input without padding, for example `320193`, so the observer also receives an invalid CIK.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Unit {
    /// A Notes period, for example `2024q1` or `2025_10`.
    Period(String),
    /// A raw CIK of the Company Facts source, for example `320193`.
    Cik(String),
    /// A date of the new-filing path, and the detection of its filings.
    Date(NaiveDate),
    /// A filing of a date of the new-filing path.
    #[serde(untagged)]
    Filing {
        /// The date of the daily index that lists the filing.
        date: NaiveDate,
        /// The accession number of the filing.
        accession_number: AccessionNumber,
    },
    /// The company of a filing of a date of the new-filing path.
    #[serde(untagged)]
    Company {
        /// The date of the daily index that lists the filing of the company.
        date: NaiveDate,
        /// The CIK of the company.
        #[serde(serialize_with = "serialize_display")]
        cik: Cik,
    },
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Period(period) => write!(f, "{period}"),
            Self::Date(date) => write!(f, "{date}"),
            Self::Filing {
                date,
                accession_number,
            } => write!(f, "{date} {accession_number}"),
            Self::Company { date, cik } => write!(f, "{date} CIK {cik}"),
            Self::Cik(cik) => write!(f, "{cik}"),
        }
    }
}

/// Serializes `value` as the string of its [`Display`].
fn serialize_display<S: Serializer>(
    value: &impl Display,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_str(value)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_display_each_kind_in_lowercase_when_formatting_error_kinds() {
        let kinds = [
            ErrorKind::Retried,
            ErrorKind::Pending,
            ErrorKind::Absent,
            ErrorKind::Unextractable,
            ErrorKind::Failed,
        ];

        let expected_result = vec!["retried", "pending", "absent", "unextractable", "failed"];

        let result: Vec<String> = kinds.iter().map(ToString::to_string).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_bare_period_when_the_unit_is_a_period() {
        let unit = Unit::Period("2024q1".to_string());

        let expected_result = "2024q1";

        let result = unit.to_string();

        assert_eq!(result, expected_result);
    }

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 26)
            .expect("Given a hardcoded calendar date, the date should always build")
    }

    fn filing() -> Unit {
        Unit::Filing {
            date: date(),
            accession_number: AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
        }
    }

    fn company() -> Unit {
        Unit::Company {
            date: date(),
            cik: Cik::new("3545")
                .expect("Given a hardcoded valid CIK, parsing should always succeed"),
        }
    }

    #[test]
    fn should_display_the_date_and_the_value_when_the_unit_is_a_daily_unit() {
        let units = [Unit::Date(date()), filing(), company()];

        let expected_result = vec![
            "2026-10-26",
            "2026-10-26 0000003545-26-000047",
            "2026-10-26 CIK 0000003545",
        ];

        let result: Vec<String> = units.iter().map(ToString::to_string).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_run_context_of_the_daily_binary_when_the_unit_is_a_daily_unit() {
        let units = [Unit::Date(date()), filing(), company()];

        let expected_result = serde_json::json!([
            { "date": "2026-10-26" },
            { "date": "2026-10-26", "accession_number": "0000003545-26-000047" },
            { "date": "2026-10-26", "cik": "0000003545" },
        ]);

        let result = serde_json::to_value(units)
            .expect("Given units of dates and plain strings, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_an_object_with_the_period_key_when_the_unit_is_a_period() {
        let unit = Unit::Period("2024q1".to_string());

        let expected_result = serde_json::json!({ "period": "2024q1" });

        let result = serde_json::to_value(&unit)
            .expect("Given a unit of plain strings, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_bare_cik_when_the_unit_is_a_cik() {
        let unit = Unit::Cik("320193".to_string());

        let expected_result = "320193";

        let result = unit.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_an_object_with_the_cik_key_when_the_unit_is_a_cik() {
        let unit = Unit::Cik("320193".to_string());

        let expected_result = serde_json::json!({ "cik": "320193" });

        let result = serde_json::to_value(&unit)
            .expect("Given a unit of plain strings, serialization should always succeed");

        assert_eq!(result, expected_result);
    }
}
