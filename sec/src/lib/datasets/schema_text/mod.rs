//! # Schema Text
//!
//! Provides [`schema_text`], the text form of a silver schema that the snapshot tests check in.

use std::fmt::Write;

use arrow::datatypes::{DataType, Field, Fields, Schema, TimeUnit};

/// Returns the text form of a schema: one line per field, with its name, type, and nullability.
///
/// `sec` prints the text, not the `Display` of Arrow, because that `Display` changes between Arrow
/// releases. A nested type prints its inner fields in the same form. A type that no silver dataset
/// uses prints as `unsupported`.
///
/// # Examples
///
/// ```rust
/// use arrow::datatypes::{DataType, Field, Schema};
/// use sec::datasets::schema_text;
///
/// let schema = Schema::new(vec![
///     Field::new("filing_id", DataType::Utf8, false),
///     Field::new("unit", DataType::Utf8, true),
/// ]);
///
/// let expected_result = "filing_id: utf8\nunit: utf8 nullable\n";
///
/// let result = schema_text(&schema);
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn schema_text(schema: &Schema) -> String {
    schema
        .fields()
        .iter()
        .fold(String::new(), |mut text, field| {
            // Deviation: the result is ignored. Writing to a `String` never fails.
            let _ = writeln!(text, "{}", field_text(field));
            text
        })
}

/// Returns the name, the type, and the nullability of one field.
fn field_text(field: &Field) -> String {
    let nullability = if field.is_nullable() { " nullable" } else { "" };
    format!(
        "{}: {}{nullability}",
        field.name(),
        type_text(field.data_type())
    )
}

/// Returns the text of one Arrow type.
fn type_text(data_type: &DataType) -> String {
    match data_type {
        DataType::Boolean => "bool".to_owned(),
        DataType::Int16 => "int16".to_owned(),
        DataType::Int32 => "int32".to_owned(),
        DataType::UInt8 => "uint8".to_owned(),
        DataType::UInt16 => "uint16".to_owned(),
        DataType::Utf8 => "utf8".to_owned(),
        DataType::Date32 => "date32".to_owned(),
        DataType::Decimal128(precision, scale) => format!("decimal128({precision}, {scale})"),
        DataType::Timestamp(unit, time_zone) => format!(
            "timestamp[{}, {}]",
            time_unit_text(*unit),
            time_zone.as_deref().unwrap_or("none")
        ),
        DataType::List(item) => format!("list<{}>", field_text(item)),
        DataType::Struct(fields) => format!("struct<{}>", fields_text(fields)),
        _ => "unsupported".to_owned(),
    }
}

/// Returns the text of the fields of a struct, separated by `, `.
fn fields_text(fields: &Fields) -> String {
    fields
        .iter()
        .map(|field| field_text(field))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Returns the text of a time unit.
const fn time_unit_text(unit: TimeUnit) -> &'static str {
    match unit {
        TimeUnit::Second => "s",
        TimeUnit::Millisecond => "ms",
        TimeUnit::Microsecond => "us",
        TimeUnit::Nanosecond => "ns",
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_print_the_inner_fields_when_a_list_holds_a_struct() {
        let pair = Fields::from(vec![
            Field::new("axis", DataType::Utf8, false),
            Field::new("member", DataType::Utf8, false),
        ]);
        let item = Field::new("item", DataType::Struct(pair), false);
        let schema = Schema::new(vec![Field::new(
            "dimensions",
            DataType::List(Arc::new(item)),
            false,
        )]);

        let expected_result = "dimensions: list<item: struct<axis: utf8, member: utf8>>\n";

        let result = schema_text(&schema);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_unit_and_time_zone_for_a_timestamp() {
        let schema = Schema::new(vec![Field::new(
            "accepted",
            DataType::Timestamp(TimeUnit::Millisecond, Some("+00:00".into())),
            true,
        )]);

        let expected_result = "accepted: timestamp[ms, +00:00] nullable\n";

        let result = schema_text(&schema);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_unsupported_for_a_type_no_dataset_uses() {
        let schema = Schema::new(vec![Field::new("ratio", DataType::Float64, false)]);

        let expected_result = "ratio: unsupported\n";

        let result = schema_text(&schema);

        assert_eq!(result, expected_result);
    }
}
