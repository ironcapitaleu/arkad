//! # Columns
//!
//! Provides the conversions between the values of a silver row and the Arrow arrays of its
//! columns, and the schema check of a batch that every row type shares.

use std::ops::Range;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayAccessor, ArrayRef, BooleanArray, Decimal128Array, Int32Array, ListArray,
    RecordBatch, StringArray, StructArray, TimestampMillisecondArray,
};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::{DataType, Field, FieldRef, Fields, Schema, TimeUnit};
use arrow::error::ArrowError;
use bigdecimal::BigDecimal;
use bigdecimal::num_bigint::BigInt;
use jiff::civil::{Date, date};
use jiff::{Span, Timestamp};
use storage::implementations::repository::filesystem::IngestionMode;
use xbrl::core::confidence::Confidence;
use xbrl::core::fact::{Decimals, Dimensions};
use xbrl::core::invalid_value::InvalidValue;
use xbrl::core::period::{Duration, Period};

use crate::datasets::constants::{DECIMAL_LIMIT, DECIMAL_PRECISION, DECIMAL_SCALE};
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

/// Returns the item field of a `dimensions` column: a struct of `axis` and `member`.
pub fn dimensions_item() -> FieldRef {
    let text = |name| Field::new(name, DataType::Utf8, false);
    let pair = Fields::from(vec![text("axis"), text("member")]);
    Arc::new(Field::new_list_field(DataType::Struct(pair), false))
}

/// Returns a `dimensions` array whose row `n` holds the pairs of the `n`-th value.
pub fn dimensions_array<'d>(values: impl Iterator<Item = &'d Dimensions> + Clone) -> ArrayRef {
    let item = dimensions_item();
    let lengths = values.clone().map(|value| value.iter().count()).collect();
    let (axes, members): (Vec<&str>, Vec<&str>) = values.flat_map(Dimensions::iter).unzip();
    let pairs = string_struct_array(
        &item,
        vec![StringArray::from(axes), StringArray::from(members)],
    );
    list_array(item, lengths, pairs)
}

/// Reads the axis and member pairs of row `row` of a `dimensions` column.
pub fn dimensions_at(array: &ListArray, row: usize) -> Result<Dimensions, InvalidValue> {
    let pairs = string_struct_list(array, row).into_iter().map(|pair| {
        let mut fields = pair.into_iter();
        // Both fields are not null in the schema, so the defaults never apply.
        (
            fields.next().flatten().unwrap_or_default(),
            fields.next().flatten().unwrap_or_default(),
        )
    });
    Dimensions::new(pairs)
}

/// Appends the axis and member pairs as a list of structs.
pub fn write_dimensions(byte_form: &mut ByteForm, dimensions: &Dimensions) {
    byte_form.list(dimensions.iter().count());
    for (axis, member) in dimensions.iter() {
        byte_form.struct_(2).utf8(axis).utf8(member);
    }
}

/// Reads the period of a row from its `period_end`, `period_start`, and `quarters` values.
///
/// Fails if a day count lies past the dates of `jiff`, if the start lies after the end, or if
/// `quarters` differs from the quarters of the dates.
pub fn period_from_columns(
    dataset: &'static str,
    row: usize,
    end_days: i32,
    start_days: Option<i32>,
    quarters: u16,
) -> Result<Period, ArrowRowError> {
    let date = |column, days| {
        date_from_days(days).map_err(|reason| unexpected_value(dataset, column, row, reason))
    };
    let end = date("period_end", end_days)?;
    let period = match start_days {
        None => Period::Instant { end },
        Some(days) => Period::Duration(
            Duration::from_dates(date("period_start", days)?, end).map_err(|source| {
                ArrowRowError::InvalidColumnValue {
                    dataset,
                    column: "period_start",
                    row,
                    source,
                }
            })?,
        ),
    };
    if period.quarters() != quarters {
        return Err(unexpected_value(
            dataset,
            "quarters",
            row,
            format!("the dates give '{}' quarters", period.quarters()),
        ));
    }
    Ok(period)
}

/// Returns a `decimal128(38, 10)` array of the numbers.
///
/// # Errors
///
/// Returns [`ArrowRowError::UnexpectedColumnValue`] if a number has no form in the column.
pub fn decimal128_array<'n>(
    dataset: &'static str,
    column: &'static str,
    values: impl Iterator<Item = Option<&'n BigDecimal>>,
) -> Result<ArrayRef, ArrowRowError> {
    let digits = values
        .enumerate()
        .map(|(row, value)| {
            value
                .map(decimal128)
                .transpose()
                .map_err(|reason| unexpected_value(dataset, column, row, reason))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let array = Decimal128Array::from(digits)
        .with_precision_and_scale(DECIMAL_PRECISION, DECIMAL_SCALE)
        .map_err(|error| failed_batch(dataset, &error))?;
    Ok(Arc::new(array))
}

/// Returns the number that the unscaled digits of a `decimal128(38, 10)` value stand for.
///
/// The number has no trailing zeros after the point: `-6.1100000000` reads as `-6.11`, and
/// `100.0000000000` as `100`. So a filed `6.10` reads back as `6.1`, an equal number.
pub fn number_from_decimal128(digits: i128) -> BigDecimal {
    let number = BigDecimal::new(BigInt::from(digits), DECIMAL_SCALE.into()).normalized();
    if number.fractional_digit_count() < 0 {
        number.with_scale(0)
    } else {
        number
    }
}

/// Returns the unscaled digits of a number in `decimal128(38, 10)`.
///
/// Fails if the number has more than 10 digits after the point, or more than 28 before it.
fn decimal128(number: &BigDecimal) -> Result<i128, String> {
    let scaled = number.with_scale(DECIMAL_SCALE.into());
    if &scaled != number {
        return Err(format!(
            "'{number}' has more than {DECIMAL_SCALE} digits after the point"
        ));
    }
    let (digits, _) = scaled.into_bigint_and_exponent();
    i128::try_from(digits)
        .ok()
        .filter(|digits| digits.unsigned_abs() < DECIMAL_LIMIT)
        .ok_or_else(|| format!("'{number}' has more than {DECIMAL_PRECISION} digits"))
}

/// Returns the `decimals` and `decimals_infinite` arrays of the precisions.
///
/// `decimals` holds the places of a finite precision, and is null otherwise. `decimals_infinite`
/// is true for an infinite precision.
pub fn decimals_arrays(values: impl Iterator<Item = Option<Decimals>> + Clone) -> [ArrayRef; 2] {
    let places = values.clone().map(|value| match value {
        Some(Decimals::Finite(places)) => Some(places),
        Some(Decimals::Infinite) | None => None,
    });
    let infinite = values.map(|value| Some(value == Some(Decimals::Infinite)));
    [
        Arc::new(Int32Array::from_iter(places)),
        Arc::new(BooleanArray::from_iter(infinite)),
    ]
}

/// Reads the precision of a row from its `decimals` and `decimals_infinite` values.
///
/// Fails if the row has finite places and `decimals_infinite` true.
pub fn decimals_from_columns(
    dataset: &'static str,
    row: usize,
    places: Option<i32>,
    infinite: bool,
) -> Result<Option<Decimals>, ArrowRowError> {
    match (places, infinite) {
        (Some(places), false) => Ok(Some(Decimals::Finite(places))),
        (None, true) => Ok(Some(Decimals::Infinite)),
        (None, false) => Ok(None),
        (Some(_), true) => Err(unexpected_value(
            dataset,
            "decimals",
            row,
            "a finite value with 'decimals_infinite' true".to_owned(),
        )),
    }
}

/// Appends the `decimals` and `decimals_infinite` values of a precision.
pub fn write_decimals(byte_form: &mut ByteForm, decimals: Option<Decimals>) {
    match decimals {
        Some(Decimals::Finite(places)) => byte_form.integer(places).bool(false),
        Some(Decimals::Infinite) => byte_form.null().bool(true),
        None => byte_form.null().bool(false),
    };
}

/// Returns the confidence whose name is `text`, for example `Exact`.
pub fn confidence_from_name(text: &str) -> Option<Confidence> {
    [
        Confidence::Exact,
        Confidence::Synonym,
        Confidence::Derived,
        Confidence::Computed,
    ]
    .into_iter()
    .find(|confidence| confidence.to_string() == text)
}

/// Returns the ingestion mode whose text is `text`: `backfill` or `incremental`.
pub fn ingestion_mode_from_name(text: &str) -> Option<IngestionMode> {
    [IngestionMode::Backfill, IngestionMode::Incremental]
        .into_iter()
        .find(|mode| mode.to_string() == text)
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

    /// Returns the array of a field of the struct items of a list column.
    ///
    /// The array holds the items of every row. The offsets of the list column give the items of
    /// one row.
    ///
    /// # Errors
    ///
    /// Returns [`ArrowRowError::UnexpectedSchema`] if the column or its field holds another array
    /// type.
    pub fn item_field<A: Array + 'static>(
        &self,
        column: &str,
        field: &str,
    ) -> Result<&'a A, ArrowRowError> {
        let list = self.get::<ListArray>(column)?;
        list.values()
            .as_any()
            .downcast_ref::<StructArray>()
            .and_then(|items| items.column_by_name(field))
            .and_then(|array| array.as_any().downcast_ref::<A>())
            .ok_or_else(|| ArrowRowError::UnexpectedSchema {
                dataset: self.dataset,
                version: self.version,
                reason: format!("field '{field}' of column '{column}' holds another array type"),
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

/// Returns the indexes of the items of row `row` in the item arrays of a list column.
pub fn item_range(array: &ListArray, row: usize) -> Range<usize> {
    let offsets = array.value_offsets();
    // The offsets of a list array never fall, and never lie past the item arrays.
    let first = usize::try_from(offsets[row]).unwrap_or_default();
    let end = usize::try_from(offsets[row + 1]).unwrap_or_default();
    first..end
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

    #[test]
    fn should_write_the_places_then_false_when_the_decimals_are_finite() {
        let decimals = Some(Decimals::Finite(-6));

        // decimals: marker 01 | length 00000002 | text -6
        // decimals_infinite: marker 01 | length 00000005 | text false
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x02-6\x01\x00\x00\x00\x05false";

        let mut byte_form = ByteForm::new();
        write_decimals(&mut byte_form, decimals);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_items_of_the_row_when_a_list_has_empty_rows() {
        let list = ListArray::new(
            Arc::new(Field::new_list_field(DataType::Utf8, false)),
            OffsetBuffer::new(vec![0, 2, 2, 5].into()),
            Arc::new(StringArray::from(vec!["a", "b", "c", "d", "e"])),
            None,
        );

        let expected_result = 2..5;

        let result = item_range(&list, 2);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_every_ingestion_mode_back_from_its_text() {
        let texts = ["backfill", "incremental"];

        let expected_result = [
            Some(IngestionMode::Backfill),
            Some(IngestionMode::Incremental),
        ];

        let result = texts.map(ingestion_mode_from_name);

        assert_eq!(result, expected_result);
    }
}
