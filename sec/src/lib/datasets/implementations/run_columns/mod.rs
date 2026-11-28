//! # Run Columns
//!
//! Provides [`RunColumns`], the columns that name the run that wrote a dataset row.

use std::sync::Arc;

use arrow::array::{ArrayRef, StringArray, TimestampMillisecondArray};
use arrow::datatypes::{DataType, Field};
use jiff::Timestamp;
use uuid::Uuid;

use crate::datasets::ArrowRowError;
use crate::datasets::columns::{
    BatchColumns, timestamp_array, timestamp_from_milliseconds, timestamp_type, unexpected_value,
};
use crate::shared::row_digest::ByteForm;

/// The version of the transform output and the run that wrote a dataset row.
///
/// Every row of a dataset ends with these columns. `sec/design/pipeline/silver_layer.md`,
/// "Provenance", names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RunColumns {
    /// The version of the transform output, as in
    /// [`TRANSFORM_VERSION`](crate::shared::transform_version::TRANSFORM_VERSION).
    pub transform_version: u32,
    /// The run that wrote the row.
    pub execution_id: Uuid,
    /// The time of the write. The part below one millisecond does not reach the batch.
    pub loaded_at: Timestamp,
}

impl RunColumns {
    /// The names of the columns, in the order of the schema.
    const COLUMNS: [&'static str; 3] = ["transform_version", "execution_id", "loaded_at"];

    /// Returns the fields of the columns, in the order of the schema.
    pub(crate) fn fields() -> Vec<Field> {
        let [version, execution, loaded] = Self::COLUMNS;
        vec![
            Field::new(version, DataType::Utf8, false),
            Field::new(execution, DataType::Utf8, false),
            Field::new(loaded, timestamp_type(), false),
        ]
    }

    /// Returns the arrays of the columns, in the order of the schema.
    pub(crate) fn arrays(rows: &[Self]) -> Vec<ArrayRef> {
        vec![
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.transform_version.to_string()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.execution_id.to_string()),
            )),
            timestamp_array(rows.iter().map(|row| Some(row.loaded_at))),
        ]
    }

    /// Reads the columns of every row of the batch.
    ///
    /// # Errors
    ///
    /// Returns [`ArrowRowError::UnexpectedColumnValue`] if a value does not parse.
    pub(crate) fn read(columns: &BatchColumns<'_>) -> Result<Vec<Self>, ArrowRowError> {
        let [version, execution, loaded] = Self::COLUMNS;
        let versions = columns.get::<StringArray>(version)?;
        let executions = columns.get::<StringArray>(execution)?;
        let loaded_ats = columns.get::<TimestampMillisecondArray>(loaded)?;
        (0..columns.row_count())
            .map(|row| {
                let fail = |column, reason: String| {
                    unexpected_value(columns.dataset(), column, row, reason)
                };
                Ok(Self {
                    transform_version: versions
                        .value(row)
                        .parse()
                        .map_err(|_| fail(version, "not a base-10 number".to_owned()))?,
                    execution_id: Uuid::parse_str(executions.value(row))
                        .map_err(|error| fail(execution, error.to_string()))?,
                    loaded_at: timestamp_from_milliseconds(loaded_ats.value(row))
                        .map_err(|reason| fail(loaded, reason))?,
                })
            })
            .collect()
    }

    /// Appends the columns that enter `content_digest`: only `transform_version`.
    ///
    /// Leaves out `execution_id` and `loaded_at`, because a rebuild of the same rows changes them.
    pub(crate) fn write_digest(&self, byte_form: &mut ByteForm) {
        byte_form.utf8(&self.transform_version.to_string());
    }
}

/// Run columns for the tests of the row types.
#[cfg(test)]
pub(crate) fn sample_run_columns() -> RunColumns {
    RunColumns {
        transform_version: 1,
        execution_id: Uuid::from_u128(1),
        loaded_at: Timestamp::from_millisecond(1_793_966_700_123).expect(
            "Given a hardcoded time in the range of jiff, the creation should always succeed",
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_run_columns() {
        implements_auto_traits::<RunColumns>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_run_columns() {
        implements_value_traits::<RunColumns>();
    }
}
