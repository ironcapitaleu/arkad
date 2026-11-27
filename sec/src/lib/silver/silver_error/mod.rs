//! # Silver Error
//!
//! Provides [`SilverError`], the error of an [`ArrowRow`](crate::silver::ArrowRow) that
//! fails to turn rows into a record batch, or a record batch into rows.

use thiserror::Error;
use xbrl::core::invalid_value::InvalidValue;

/// Error indicating that rows of a silver dataset and their record batch do not match.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SilverError {
    /// The dataset has no schema of this version.
    #[error("[UnknownSchemaVersion] Dataset '{dataset}' has no schema version '{version}'")]
    UnknownSchemaVersion {
        /// The dataset, for example `as_is_facts`.
        dataset: &'static str,
        /// The schema version of the unit.
        version: u16,
    },

    /// The fields of the batch differ from the schema of its version.
    #[error(
        "[UnexpectedSchema] The batch fields differ from schema version '{version}' of dataset '{dataset}', Reason: '{reason}'"
    )]
    UnexpectedSchema {
        /// The dataset, for example `as_is_facts`.
        dataset: &'static str,
        /// The schema version of the unit.
        version: u16,
        /// The first field that differs, in the form of
        /// [`schema_text`](crate::silver::schema_text()).
        reason: String,
    },

    /// A value of the batch breaks a rule of its `xbrl` type.
    #[error(
        "[InvalidColumnValue] Failed to read row '{row}' of column '{column}' in dataset '{dataset}', Caused by: {source}"
    )]
    InvalidColumnValue {
        /// The dataset, for example `as_is_facts`.
        dataset: &'static str,
        /// The column, for example `unit`.
        column: &'static str,
        /// The index of the row in the batch.
        row: usize,
        /// The rule the value breaks.
        #[source]
        source: InvalidValue,
    },

    /// A value has no form in its column, or the values of a row contradict each other.
    #[error(
        "[UnexpectedColumnValue] Row '{row}' of column '{column}' in dataset '{dataset}' holds an unexpected value, Reason: '{reason}'"
    )]
    UnexpectedColumnValue {
        /// The dataset, for example `as_is_facts`.
        dataset: &'static str,
        /// The column, for example `value_numeric`.
        column: &'static str,
        /// The index of the row in the rows or in the batch.
        row: usize,
        /// Why the value has no form in the column.
        reason: String,
    },

    /// Arrow rejects the columns of the batch.
    #[error(
        "[FailedBatchCreation] Failed to build the record batch of dataset '{dataset}', Reason: '{reason}'"
    )]
    FailedBatchCreation {
        /// The dataset, for example `as_is_facts`.
        dataset: &'static str,
        /// The text of the Arrow error.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;
    use xbrl::core::invalid_value::InvalidValueReason;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_silver_error() {
        implements_auto_traits::<SilverError>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_silver_error() {
        implements_value_traits::<SilverError>();
    }

    #[test]
    fn should_chain_the_rule_when_a_column_value_is_invalid() {
        let error = SilverError::InvalidColumnValue {
            dataset: "as_is_facts",
            column: "unit",
            row: 3,
            source: InvalidValue::new("Unit", InvalidValueReason::Empty, ""),
        };

        let expected_result = "[InvalidColumnValue] Failed to read row '3' of column 'unit' in dataset 'as_is_facts', Caused by: [InvalidValue] Invalid 'Unit', Reason: 'The value is empty', Input: ''";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
