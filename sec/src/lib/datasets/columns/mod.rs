//! # Columns
//!
//! Provides the conversions between the values of a silver row and the Arrow arrays of its
//! columns, and the schema check of a batch that every row type shares.

use std::sync::Arc;

use arrow::array::{
    Array, ArrayAccessor, ArrayRef, ListArray, RecordBatch, StringArray, StructArray,
    TimestampMillisecondArray,
};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::{DataType, Field, FieldRef, Fields, Schema, TimeUnit};
use arrow::error::ArrowError;
use chrono::{DateTime, NaiveDate, Utc};
use jiff::civil::{Date, date};
use jiff::{Span, Timestamp};

use crate::datasets::{ArrowRow, ArrowRowError, UTC_TIME_ZONE, schema_text};
use crate::shared::row_digest::ByteForm;

/// The date that `date32` counts its days from.
const UNIX_EPOCH: Date = date(1970, 1, 1);

/// Returns the type of a `timestamp[ms, +00:00]` column.
pub fn timestamp_type() -> DataType {
    DataType::Timestamp(TimeUnit::Millisecond, Some(UTC_TIME_ZONE.into()))
}

/// Returns the type of a list whose items are of `item_type` and never null.
pub fn list_type(item_type: DataType) -> DataType {
    DataType::new_list(item_type, false)
}

/// Returns the item field of a list column of the schema.
pub fn list_item(schema: &Schema, column: &str) -> FieldRef {
    match schema.field_with_name(column).map(Field::data_type) {
        Ok(DataType::List(item)) => Arc::clone(item),
        _ => unreachable!("Given a column that the schema of its dataset holds as a list"),
    }
}

/// Returns the days from 1970-01-01 to the date, the value of a `date32` column.
pub fn date32(value: Date) -> i32 {
    (value - UNIX_EPOCH).get_days()
}

/// Returns the date `days` after 1970-01-01.
pub fn date_from_days(days: i32) -> Result<Date, String> {
    Span::new()
        .try_days(days)
        .and_then(|span| UNIX_EPOCH.checked_add(span))
        .map_err(|error| error.to_string())
}

/// Returns the timestamp `milliseconds` after the Unix epoch.
pub fn timestamp_from_milliseconds(milliseconds: i64) -> Result<Timestamp, String> {
    Timestamp::from_millisecond(milliseconds).map_err(|error| error.to_string())
}

/// Returns a `timestamp[ms, +00:00]` array of the timestamps. The part below one millisecond drops.
pub fn timestamp_array(values: impl IntoIterator<Item = Option<Timestamp>>) -> ArrayRef {
    let milliseconds = values
        .into_iter()
        .map(|value| value.map(Timestamp::as_millisecond));
    Arc::new(TimestampMillisecondArray::from_iter(milliseconds).with_timezone(UTC_TIME_ZONE))
}

/// Returns a list array whose row `n` holds the next `lengths[n]` items of `items`.
pub fn list_array(item: FieldRef, lengths: Vec<usize>, items: ArrayRef) -> ArrayRef {
    Arc::new(ListArray::new(
        item,
        OffsetBuffer::from_lengths(lengths),
        items,
        None,
    ))
}

/// Returns a struct array of the string columns, one per field of the struct item of `item`.
pub fn string_struct_array(item: &FieldRef, columns: Vec<StringArray>) -> ArrayRef {
    let DataType::Struct(fields) = item.data_type() else {
        unreachable!("Given the item field of a list of structs in a silver schema")
    };
    let columns = columns
        .into_iter()
        .map(|column| Arc::new(column) as ArrayRef)
        .collect();
    Arc::new(StructArray::new(Fields::clone(fields), columns, None))
}

/// Returns the value of row `row`, or `None` if it is null.
pub fn nullable<A: ArrayAccessor + Copy>(array: A, row: usize) -> Option<A::Item> {
    array.is_valid(row).then(|| array.value(row))
}

/// Returns the date as a `chrono` date, the type of [`ByteForm::date`](crate::shared::row_digest::ByteForm::date).
pub fn naive_date(value: Date) -> NaiveDate {
    // Deviation: `ByteForm` still takes `chrono` dates, because ADR 0047 moves `sec` in a later
    // slice. Every `jiff` date lies in the range of `chrono`, so the conversion never fails.
    NaiveDate::from_ymd_opt(
        i32::from(value.year()),
        u32::from(value.month().unsigned_abs()),
        u32::from(value.day().unsigned_abs()),
    )
    .expect("Given a valid jiff date, the creation of the same chrono date should always succeed")
}

/// Returns the timestamp as a `chrono` time, the type of [`ByteForm::timestamp`](crate::shared::row_digest::ByteForm::timestamp).
pub fn utc_time(value: Timestamp) -> DateTime<Utc> {
    // Deviation: the reason of `naive_date` holds here too.
    DateTime::from_timestamp_millis(value.as_millisecond()).expect(
        "Given a valid jiff timestamp, the creation of the same chrono time should always succeed",
    )
}

/// Reads the arrays of a batch whose fields match the schema of a version of `R`.
pub struct BatchColumns<'a> {
    batch: &'a RecordBatch,
    dataset: &'static str,
    version: u16,
}

impl<'a> BatchColumns<'a> {
    /// Checks the fields of the batch against the schema of `version` of `R`.
    ///
    /// # Errors
    ///
    /// Returns [`ArrowRowError::UnknownSchemaVersion`] if `R` has no schema of `version`, and
    /// [`ArrowRowError::UnexpectedSchema`] if a field of the batch differs from the schema.
    pub fn check<R: ArrowRow>(batch: &'a RecordBatch, version: u16) -> Result<Self, ArrowRowError> {
        let schema = R::schema(version).ok_or(ArrowRowError::UnknownSchemaVersion {
            dataset: R::DATASET,
            version,
        })?;
        let expected = schema_text(&schema);
        let found = schema_text(&batch.schema());
        let mut expected_lines = expected.lines();
        let mut found_lines = found.lines();
        loop {
            match (expected_lines.next(), found_lines.next()) {
                (None, None) => break,
                (expected, found) if expected == found => {}
                (expected, found) => {
                    return Err(ArrowRowError::UnexpectedSchema {
                        dataset: R::DATASET,
                        version,
                        reason: format!(
                            "expected field '{}', found '{}'",
                            expected.unwrap_or("none"),
                            found.unwrap_or("none")
                        ),
                    });
                }
            }
        }
        let columns = Self {
            batch,
            dataset: R::DATASET,
            version,
        };
        Ok(columns)
    }

    /// Returns the array of a column as its Arrow array type.
    ///
    /// # Errors
    ///
    /// Returns [`ArrowRowError::UnexpectedSchema`] if the column holds another array type.
    pub fn get<A: Array + 'static>(&self, column: &str) -> Result<&'a A, ArrowRowError> {
        self.batch
            .column_by_name(column)
            .and_then(|array| array.as_any().downcast_ref::<A>())
            .ok_or_else(|| ArrowRowError::UnexpectedSchema {
                dataset: self.dataset,
                version: self.version,
                reason: format!("column '{column}' holds another array type"),
            })
    }

    /// Returns the dataset of the batch.
    pub const fn dataset(&self) -> &'static str {
        self.dataset
    }

    /// Returns the rows of the batch.
    pub fn row_count(&self) -> usize {
        self.batch.num_rows()
    }
}

/// Returns an [`ArrowRowError::FailedBatchCreation`] for an Arrow error.
pub fn failed_batch(dataset: &'static str, error: &ArrowError) -> ArrowRowError {
    ArrowRowError::FailedBatchCreation {
        dataset,
        reason: error.to_string(),
    }
}

/// Appends the text to the byte form, or a null if there is none.
pub fn utf8_or_null(byte_form: &mut ByteForm, value: Option<&str>) {
    match value {
        Some(text) => byte_form.utf8(text),
        None => byte_form.null(),
    };
}

/// Returns an [`ArrowRowError::UnexpectedColumnValue`] for a value of a row.
pub const fn unexpected_value(
    dataset: &'static str,
    column: &'static str,
    row: usize,
    reason: String,
) -> ArrowRowError {
    ArrowRowError::UnexpectedColumnValue {
        dataset,
        column,
        row,
        reason,
    }
}

/// Returns the strings of row `row` of a list column of strings.
pub fn string_list(array: &ListArray, row: usize) -> Vec<String> {
    let items = array.value(row);
    string_column(&items)
        .iter()
        .flatten()
        .map(str::to_owned)
        .collect()
}

/// Returns the string columns of the struct items of row `row` of a list column.
pub fn string_struct_list(array: &ListArray, row: usize) -> Vec<Vec<Option<String>>> {
    let items = array.value(row);
    let Some(items) = items.as_any().downcast_ref::<StructArray>() else {
        unreachable!("Given a list column of structs that passed the schema check")
    };
    (0..items.len())
        .map(|item| {
            items
                .columns()
                .iter()
                .map(|field| nullable(string_column(field), item).map(str::to_owned))
                .collect()
        })
        .collect()
}

/// Returns an array of a column that the schema check found to be `utf8` as a string array.
fn string_column(array: &ArrayRef) -> &StringArray {
    let Some(strings) = array.as_any().downcast_ref::<StringArray>() else {
        unreachable!("Given a utf8 array that passed the schema check")
    };
    strings
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_count_days_from_1970_when_a_date_is_later() {
        let value = date(2026, 11, 6);

        let expected_result = 20_763;

        let result = date32(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_date_of_its_day_count_when_a_date_reads_back() {
        let days = date32(date(1903, 2, 28));

        let expected_result = Ok(date(1903, 2, 28));

        let result = date_from_days(days);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_day_count_lies_past_the_dates_of_jiff() {
        let days = i32::MAX;

        // The text of the `jiff` error is not stable across releases, so the test checks only
        // that the call fails.
        let expected_result = true;

        let result = date_from_days(days).is_err();

        assert_eq!(result, expected_result);
    }
}
