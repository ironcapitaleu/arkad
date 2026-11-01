//! # Run Event
//!
//! Provides [`RunEvent`], [`ErrorKind`], and [`Unit`], the values that a batch run passes to its
//! observer.
//!
//! A batch run calls the observer once for each stream item and once for each error, at the
//! moment the machine emits it. So the caller can log during the run, not only after it.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use crate::prelude::{StreamError, StreamItem};

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
/// [`Display`] gives the bare value, for example `2024q1`. [`Serialize`] gives an object with the
/// kind of the unit as its key, for example `{"period": "2024q1"}`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    /// A Notes period, for example `2024q1` or `2025_10`.
    Period(String),
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Period(period) => write!(f, "{period}"),
        }
    }
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

    #[test]
    fn should_serialize_an_object_with_the_period_key_when_the_unit_is_a_period() {
        let unit = Unit::Period("2024q1".to_string());

        let expected_result = serde_json::json!({ "period": "2024q1" });

        let result = serde_json::to_value(&unit)
            .expect("Given a unit of plain strings, serialization should always succeed");

        assert_eq!(result, expected_result);
    }
}
