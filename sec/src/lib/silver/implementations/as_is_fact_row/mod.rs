//! # As-Is Fact Row
//!
//! Provides [`AsIsFactRow`], one row of the silver dataset `as_is_facts`: an [`AsIsFact`] and its
//! [`Provenance`].

use std::str::FromStr;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, BooleanArray, Date32Array, Decimal128Array, Int32Array, ListArray, RecordBatch,
    StringArray, TimestampMillisecondArray, UInt16Array,
};
use arrow::datatypes::{DataType, Field, Fields, Schema, SchemaRef};
use bigdecimal::BigDecimal;
use bigdecimal::num_bigint::BigInt;
use xbrl::core::concept::{ConceptName, NamespaceFamily};
use xbrl::core::fact::{
    AsIsFact, Context, Decimals, Dimensions, FactValue, FilingSource, TaxonomyVersion,
};
use xbrl::core::filing::{EntityId, FilingId};
use xbrl::core::invalid_value::InvalidValue;
use xbrl::core::period::{Duration, Period};
use xbrl::core::unit::Unit;

use self::constants::{
    DECIMAL_LIMIT, DECIMAL_PRECISION, DECIMAL_SCALE, VALUE_KIND_NIL, VALUE_KIND_NUMERIC,
    VALUE_KIND_TEXT,
};
use crate::shared::row_digest::{ByteForm, DigestRow};
use crate::silver::columns::{
    BatchColumns, date_from_days, date32, failed_batch, list_array, list_item, list_type,
    naive_date, nullable, string_list, string_struct_array, string_struct_list, timestamp_array,
    timestamp_from_milliseconds, timestamp_type, unexpected_value, utc_time, utf8_or_null,
};
use crate::silver::{Provenance, SilverError, SilverRecord};

pub mod constants;

/// One row of `as_is_facts`: a reported value, its filing, and the raw document it comes from.
///
/// `sec/design/pipeline/silver_layer.md`, `as_is_facts`, names the columns and their Arrow types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AsIsFactRow {
    /// The fact as the source reports it.
    pub fact: AsIsFact,
    /// The raw document of the fact, and the run that wrote the row.
    pub provenance: Provenance,
}

impl SilverRecord for AsIsFactRow {
    const DATASET: &'static str = "as_is_facts";
    const SCHEMA_VERSION: u16 = 1;

    fn schema(version: u16) -> Option<SchemaRef> {
        (version == 1).then(schema_v1)
    }

    fn to_batch(rows: &[Self]) -> Result<RecordBatch, SilverError> {
        let schema = schema_v1();
        let facts = || rows.iter().map(|row| &row.fact);
        let text = |value: fn(&AsIsFact) -> Option<String>| -> ArrayRef {
            Arc::new(StringArray::from_iter(facts().map(value)))
        };
        let numbers = facts()
            .enumerate()
            .map(|(row, fact)| match &fact.value {
                FactValue::Numeric(number) => decimal128(number).map(Some).map_err(|reason| {
                    unexpected_value(Self::DATASET, "value_numeric", row, reason)
                }),
                FactValue::Text(_) | FactValue::Nil => Ok(None),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let numbers = Decimal128Array::from(numbers)
            .with_precision_and_scale(DECIMAL_PRECISION, DECIMAL_SCALE)
            .map_err(|error| failed_batch(Self::DATASET, &error))?;
        let dimension_item = list_item(&schema, "dimensions");
        let (axes, members): (Vec<&str>, Vec<&str>) = facts()
            .flat_map(|fact| fact.context.dimensions.iter())
            .unzip();
        let footnote_item = list_item(&schema, "footnotes");
        let footnotes = facts().flat_map(|fact| fact.footnotes.iter());
        let mut columns: Vec<ArrayRef> = vec![
            text(|fact| Some(fact.source.filing.to_string())),
            text(|fact| Some(fact.context.entity.scheme().to_owned())),
            text(|fact| Some(fact.context.entity.id().to_owned())),
            text(|fact| Some(fact.concept.family().to_string())),
            text(|fact| Some(fact.concept.local_name().to_owned())),
            text(|fact| fact.taxonomy_version.as_ref().map(ToString::to_string)),
            Arc::new(Date32Array::from_iter_values(
                facts().map(|fact| date32(fact.context.period.end())),
            )),
            Arc::new(Date32Array::from_iter(
                facts().map(|fact| fact.context.period.start().map(date32)),
            )),
            Arc::new(UInt16Array::from_iter_values(
                facts().map(|fact| fact.context.period.quarters()),
            )),
            text(|fact| fact.unit.as_ref().map(ToString::to_string)),
            list_array(
                Arc::clone(&dimension_item),
                facts()
                    .map(|fact| fact.context.dimensions.iter().count())
                    .collect(),
                string_struct_array(
                    &dimension_item,
                    vec![StringArray::from(axes), StringArray::from(members)],
                ),
            ),
            text(|fact| Some(value_kind(&fact.value).to_owned())),
            Arc::new(numbers),
            text(|fact| match &fact.value {
                FactValue::Text(text) => Some(text.clone()),
                FactValue::Numeric(_) | FactValue::Nil => None,
            }),
            Arc::new(Int32Array::from_iter(facts().map(
                |fact| match fact.decimals {
                    Some(Decimals::Finite(places)) => Some(places),
                    Some(Decimals::Infinite) | None => None,
                },
            ))),
            Arc::new(BooleanArray::from_iter(
                facts().map(|fact| Some(fact.decimals == Some(Decimals::Infinite))),
            )),
            Arc::new(UInt16Array::from_iter_values(
                facts().map(|fact| fact.copy_rank),
            )),
            list_array(
                footnote_item,
                facts().map(|fact| fact.footnotes.len()).collect(),
                Arc::new(StringArray::from_iter_values(footnotes)),
            ),
            Arc::new(Date32Array::from_iter_values(
                facts().map(|fact| date32(fact.source.filed)),
            )),
            timestamp_array(facts().map(|fact| fact.source.accepted)),
        ];
        let provenances: Vec<&Provenance> = rows.iter().map(|row| &row.provenance).collect();
        columns.extend(Provenance::arrays(&provenances));
        RecordBatch::try_new(schema, columns).map_err(|error| failed_batch(Self::DATASET, &error))
    }

    fn from_batch(batch: &RecordBatch, version: u16) -> Result<Vec<Self>, SilverError> {
        let (columns, _) = BatchColumns::check::<Self>(batch, version)?;
        let provenances = Provenance::read(&columns)?;
        let reader = FactColumns::new(&columns)?;
        provenances
            .into_iter()
            .enumerate()
            .map(|(row, provenance)| {
                Ok(Self {
                    fact: reader.fact(row)?,
                    provenance,
                })
            })
            .collect()
    }
}

impl DigestRow for AsIsFactRow {
    fn write_group_id(&self, byte_form: &mut ByteForm) {
        byte_form.utf8(&self.fact.source.filing.to_string());
    }

    /// Appends the value key: `filing_id`, `concept_family`, `concept_name`, `taxonomy_version`,
    /// `period_end`, `quarters`, `unit`, `dimensions`, and `copy_rank`.
    fn write_row_key(&self, byte_form: &mut ByteForm) {
        let fact = &self.fact;
        byte_form
            .utf8(&fact.source.filing.to_string())
            .utf8(fact.concept.family().as_str())
            .utf8(fact.concept.local_name());
        utf8_or_null(
            byte_form,
            fact.taxonomy_version.as_ref().map(TaxonomyVersion::as_str),
        );
        byte_form
            .date(naive_date(fact.context.period.end()))
            .integer(fact.context.period.quarters());
        utf8_or_null(
            byte_form,
            fact.unit.as_ref().map(ToString::to_string).as_deref(),
        );
        write_dimensions(byte_form, &fact.context.dimensions);
        byte_form.integer(fact.copy_rank);
    }

    fn write_row(&self, byte_form: &mut ByteForm) {
        let fact = &self.fact;
        byte_form
            .utf8(&fact.source.filing.to_string())
            .utf8(fact.context.entity.scheme())
            .utf8(fact.context.entity.id())
            .utf8(fact.concept.family().as_str())
            .utf8(fact.concept.local_name());
        utf8_or_null(
            byte_form,
            fact.taxonomy_version.as_ref().map(TaxonomyVersion::as_str),
        );
        byte_form.date(naive_date(fact.context.period.end()));
        match fact.context.period.start() {
            Some(start) => byte_form.date(naive_date(start)),
            None => byte_form.null(),
        };
        byte_form.integer(fact.context.period.quarters());
        utf8_or_null(
            byte_form,
            fact.unit.as_ref().map(ToString::to_string).as_deref(),
        );
        write_dimensions(byte_form, &fact.context.dimensions);
        byte_form.utf8(value_kind(&fact.value));
        match &fact.value {
            FactValue::Numeric(number) => byte_form.decimal(number).null(),
            FactValue::Text(text) => byte_form.null().utf8(text),
            FactValue::Nil => byte_form.null().null(),
        };
        match fact.decimals {
            Some(Decimals::Finite(places)) => byte_form.integer(places).bool(false),
            Some(Decimals::Infinite) => byte_form.null().bool(true),
            None => byte_form.null().bool(false),
        };
        byte_form.integer(fact.copy_rank).list(fact.footnotes.len());
        for footnote in &fact.footnotes {
            byte_form.utf8(footnote);
        }
        byte_form.date(naive_date(fact.source.filed));
        match fact.source.accepted {
            Some(accepted) => byte_form.timestamp(utc_time(accepted)),
            None => byte_form.null(),
        };
        self.provenance.write_digest(byte_form);
    }
}

/// Returns the schema of version 1.
fn schema_v1() -> SchemaRef {
    let text = |name: &str, nullable: bool| Field::new(name, DataType::Utf8, nullable);
    let date = |name: &str, nullable: bool| Field::new(name, DataType::Date32, nullable);
    let pair = Fields::from(vec![text("axis", false), text("member", false)]);
    let mut fields = vec![
        text("filing_id", false),
        text("entity_scheme", false),
        text("entity_id", false),
        text("concept_family", false),
        text("concept_name", false),
        text("taxonomy_version", true),
        date("period_end", false),
        date("period_start", true),
        Field::new("quarters", DataType::UInt16, false),
        text("unit", true),
        Field::new("dimensions", list_type(DataType::Struct(pair)), false),
        text("value_kind", false),
        Field::new(
            "value_numeric",
            DataType::Decimal128(DECIMAL_PRECISION, DECIMAL_SCALE),
            true,
        ),
        text("value_text", true),
        Field::new("decimals", DataType::Int32, true),
        Field::new("decimals_infinite", DataType::Boolean, false),
        Field::new("copy_rank", DataType::UInt16, false),
        Field::new("footnotes", list_type(DataType::Utf8), false),
        date("filed", false),
        Field::new("accepted", timestamp_type(), true),
    ];
    fields.extend(Provenance::fields());
    Arc::new(Schema::new(fields))
}

/// The arrays of the fact columns of one batch.
struct FactColumns<'a> {
    dataset: &'static str,
    filing_ids: &'a StringArray,
    entity_schemes: &'a StringArray,
    entity_ids: &'a StringArray,
    concept_families: &'a StringArray,
    concept_names: &'a StringArray,
    taxonomy_versions: &'a StringArray,
    period_ends: &'a Date32Array,
    period_starts: &'a Date32Array,
    quarters: &'a UInt16Array,
    units: &'a StringArray,
    dimensions: &'a ListArray,
    value_kinds: &'a StringArray,
    value_numerics: &'a Decimal128Array,
    value_texts: &'a StringArray,
    decimals: &'a Int32Array,
    decimals_infinite: &'a BooleanArray,
    copy_ranks: &'a UInt16Array,
    footnotes: &'a ListArray,
    fileds: &'a Date32Array,
    accepteds: &'a TimestampMillisecondArray,
}

impl<'a> FactColumns<'a> {
    /// Takes the arrays of the fact columns from a batch that passed the schema check.
    fn new(columns: &BatchColumns<'a>) -> Result<Self, SilverError> {
        Ok(Self {
            dataset: columns.dataset(),
            filing_ids: columns.get("filing_id")?,
            entity_schemes: columns.get("entity_scheme")?,
            entity_ids: columns.get("entity_id")?,
            concept_families: columns.get("concept_family")?,
            concept_names: columns.get("concept_name")?,
            taxonomy_versions: columns.get("taxonomy_version")?,
            period_ends: columns.get("period_end")?,
            period_starts: columns.get("period_start")?,
            quarters: columns.get("quarters")?,
            units: columns.get("unit")?,
            dimensions: columns.get("dimensions")?,
            value_kinds: columns.get("value_kind")?,
            value_numerics: columns.get("value_numeric")?,
            value_texts: columns.get("value_text")?,
            decimals: columns.get("decimals")?,
            decimals_infinite: columns.get("decimals_infinite")?,
            copy_ranks: columns.get("copy_rank")?,
            footnotes: columns.get("footnotes")?,
            fileds: columns.get("filed")?,
            accepteds: columns.get("accepted")?,
        })
    }

    /// Reads the fact of one row.
    fn fact(&self, row: usize) -> Result<AsIsFact, SilverError> {
        let invalid = |column: &'static str| {
            move |source: InvalidValue| SilverError::InvalidColumnValue {
                dataset: self.dataset,
                column,
                row,
                source,
            }
        };
        let unexpected = |column: &'static str| {
            move |reason| unexpected_value(self.dataset, column, row, reason)
        };
        let date = |column: &'static str, days| date_from_days(days).map_err(unexpected(column));
        let period_end = date("period_end", self.period_ends.value(row))?;
        let period = match nullable(self.period_starts, row) {
            None => Period::Instant { end: period_end },
            Some(days) => Period::Duration(
                Duration::from_dates(date("period_start", days)?, period_end)
                    .map_err(invalid("period_start"))?,
            ),
        };
        if period.quarters() != self.quarters.value(row) {
            return Err(unexpected("quarters")(format!(
                "the dates give '{}' quarters",
                period.quarters()
            )));
        }
        let pairs = string_struct_list(self.dimensions, row)
            .into_iter()
            .map(|pair| {
                let mut fields = pair.into_iter().flatten();
                // Both fields are not null in the schema, so the defaults never apply.
                (
                    fields.next().unwrap_or_default(),
                    fields.next().unwrap_or_default(),
                )
            });
        Ok(AsIsFact {
            source: FilingSource {
                filing: FilingId::from_str(self.filing_ids.value(row))
                    .map_err(invalid("filing_id"))?,
                filed: date("filed", self.fileds.value(row))?,
                accepted: nullable(self.accepteds, row)
                    .map(timestamp_from_milliseconds)
                    .transpose()
                    .map_err(unexpected("accepted"))?,
            },
            context: Context {
                entity: EntityId::new(self.entity_schemes.value(row), self.entity_ids.value(row))
                    .map_err(invalid("entity_id"))?,
                period,
                dimensions: Dimensions::new(pairs).map_err(invalid("dimensions"))?,
            },
            concept: ConceptName::new(
                NamespaceFamily::new(self.concept_families.value(row))
                    .map_err(invalid("concept_family"))?,
                self.concept_names.value(row),
            )
            .map_err(invalid("concept_name"))?,
            taxonomy_version: nullable(self.taxonomy_versions, row)
                .map(TaxonomyVersion::new)
                .transpose()
                .map_err(invalid("taxonomy_version"))?,
            unit: nullable(self.units, row)
                .map(Unit::from_str)
                .transpose()
                .map_err(invalid("unit"))?,
            value: self.value(row)?,
            decimals: match (
                nullable(self.decimals, row),
                self.decimals_infinite.value(row),
            ) {
                (Some(places), false) => Some(Decimals::Finite(places)),
                (None, true) => Some(Decimals::Infinite),
                (None, false) => None,
                (Some(_), true) => {
                    return Err(unexpected("decimals")(
                        "a finite value with 'decimals_infinite' true".to_owned(),
                    ));
                }
            },
            copy_rank: self.copy_ranks.value(row),
            footnotes: string_list(self.footnotes, row),
        })
    }

    /// Reads the value of one row from `value_kind` and the value column of its kind.
    fn value(&self, row: usize) -> Result<FactValue, SilverError> {
        let kind = self.value_kinds.value(row);
        let value = match kind {
            VALUE_KIND_NUMERIC => nullable(self.value_numerics, row).map(|digits| {
                FactValue::Numeric(BigDecimal::new(BigInt::from(digits), DECIMAL_SCALE.into()))
            }),
            VALUE_KIND_TEXT => {
                nullable(self.value_texts, row).map(|text| FactValue::Text(text.to_owned()))
            }
            VALUE_KIND_NIL => Some(FactValue::Nil),
            _ => None,
        };
        value.ok_or_else(|| {
            unexpected_value(
                self.dataset,
                "value_kind",
                row,
                format!("kind '{kind}' without a value in its column"),
            )
        })
    }
}

/// Returns the text of the `value_kind` column of a value.
const fn value_kind(value: &FactValue) -> &'static str {
    match value {
        FactValue::Numeric(_) => VALUE_KIND_NUMERIC,
        FactValue::Text(_) => VALUE_KIND_TEXT,
        FactValue::Nil => VALUE_KIND_NIL,
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

/// Appends the axis and member pairs as a list of structs.
fn write_dimensions(byte_form: &mut ByteForm, dimensions: &Dimensions) {
    byte_form.list(dimensions.iter().count());
    for (axis, member) in dimensions.iter() {
        byte_form.struct_(2).utf8(axis).utf8(member);
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use jiff::Timestamp;
    use jiff::civil::date;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::silver::implementations::provenance::sample_provenance;
    use crate::silver::{FOLDER_LABELS, schema_text};

    /// A fact of the filing `sec:0000320193-26-000001` with the given value and precision.
    fn fact(value: FactValue, decimals: Option<Decimals>) -> AsIsFact {
        let parse = "Given hardcoded valid parts, the creation should always succeed";
        AsIsFact {
            source: FilingSource {
                filing: FilingId::from_str("sec:0000320193-26-000001").expect(parse),
                filed: date(2026, 11, 6),
                accepted: Some(Timestamp::from_millisecond(1_793_966_700_000).expect(parse)),
            },
            context: Context {
                entity: EntityId::new("http://www.sec.gov/CIK", "0000320193").expect(parse),
                period: Period::Duration(
                    Duration::from_dates(date(2025, 9, 28), date(2026, 9, 26)).expect(parse),
                ),
                dimensions: Dimensions::new([("StatementBusinessSegmentsAxis", "IPhoneMember")])
                    .expect(parse),
            },
            concept: ConceptName::new(NamespaceFamily::new("us-gaap").expect(parse), "Revenues")
                .expect(parse),
            taxonomy_version: Some(TaxonomyVersion::new("us-gaap/2026").expect(parse)),
            unit: Some(Unit::from_str("USD").expect(parse)),
            value,
            decimals,
            copy_rank: 0,
            footnotes: vec!["Restated, see note 2".to_owned()],
        }
    }

    /// A number with 28 digits before the point and 10 after it.
    fn number(text: &str) -> FactValue {
        FactValue::Numeric(
            BigDecimal::from_str(text)
                .expect("Given a hardcoded decimal text, the parse should always succeed"),
        )
    }

    /// The fixture rows: 38 digits, non-ASCII text, infinite decimals, and every nullable field
    /// empty.
    fn sample_rows() -> Vec<AsIsFactRow> {
        let row = |fact| AsIsFactRow {
            fact,
            provenance: sample_provenance(),
        };
        let mut empty = fact(FactValue::Nil, None);
        empty.source.accepted = None;
        empty.context.period = Period::Instant {
            end: date(2026, 9, 26),
        };
        empty.context.dimensions = Dimensions::default();
        empty.taxonomy_version = None;
        empty.unit = None;
        empty.footnotes = Vec::new();
        vec![
            row(fact(
                number("1234567890123456789012345678.1234567890"),
                Some(Decimals::Finite(10)),
            )),
            row(fact(
                FactValue::Text("Société Générale — 株式会社 ✓".to_owned()),
                None,
            )),
            row(fact(number("-6.11"), Some(Decimals::Infinite))),
            row(empty),
        ]
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_as_is_fact_row() {
        implements_auto_traits::<AsIsFactRow>();
    }

    const fn implements_value_traits<T: Debug + Clone + PartialEq + Eq + Hash>() {}
    #[test]
    const fn should_implement_value_traits_for_as_is_fact_row() {
        implements_value_traits::<AsIsFactRow>();
    }

    #[test]
    fn should_return_equal_rows_when_rows_pass_through_a_batch() {
        let rows = sample_rows();

        let expected_result = sample_rows();

        let batch = AsIsFactRow::to_batch(&rows)
            .expect("Given rows whose values fit their columns, the batch should always build");
        let result = AsIsFactRow::from_batch(&batch, AsIsFactRow::SCHEMA_VERSION)
            .expect("Given a batch that `to_batch` built, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_match_the_checked_in_snapshot_for_the_schema_of_version_1() {
        let schema = AsIsFactRow::schema(1).expect("Given version 1, the schema should exist");

        let expected_result = include_str!("schema_v1.txt");

        let result = schema_text(&schema);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_no_field_after_a_folder_label_for_as_is_facts() {
        let schema = AsIsFactRow::schema(AsIsFactRow::SCHEMA_VERSION)
            .expect("Given the version that `to_batch` writes, the schema should exist");

        let expected_result: Vec<&str> = Vec::new();

        let result: Vec<&str> = schema
            .fields()
            .iter()
            .map(|field| field.name().as_str())
            .filter(|name| FOLDER_LABELS.contains(name))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_schema_version_is_unknown() {
        let batch = AsIsFactRow::to_batch(&sample_rows())
            .expect("Given rows whose values fit their columns, the batch should always build");

        let expected_result = Err(SilverError::UnknownSchemaVersion {
            dataset: "as_is_facts",
            version: 2,
        });

        let result = AsIsFactRow::from_batch(&batch, 2);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_batch_lacks_a_field_of_the_schema() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "filing_id",
            DataType::Utf8,
            false,
        )]));
        let column: ArrayRef = Arc::new(StringArray::from(vec!["sec:0000320193-26-000001"]));
        let batch = RecordBatch::try_new(schema, vec![column])
            .expect("Given one utf8 column for one utf8 field, the batch should always build");

        let expected_result = Err(SilverError::UnexpectedSchema {
            dataset: "as_is_facts",
            version: 1,
            reason: "expected field 'entity_scheme: utf8', found 'none'".to_owned(),
        });

        let result = AsIsFactRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_number_has_more_than_10_digits_after_the_point() {
        let rows = vec![AsIsFactRow {
            fact: fact(number("0.12345678901"), None),
            provenance: sample_provenance(),
        }];

        let expected_result = Err(SilverError::UnexpectedColumnValue {
            dataset: "as_is_facts",
            column: "value_numeric",
            row: 0,
            reason: "'0.12345678901' has more than 10 digits after the point".to_owned(),
        });

        let result = AsIsFactRow::to_batch(&rows);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_number_has_more_than_28_digits_before_the_point() {
        let rows = vec![AsIsFactRow {
            fact: fact(number("12345678901234567890123456789"), None),
            provenance: sample_provenance(),
        }];

        let expected_result = Err(SilverError::UnexpectedColumnValue {
            dataset: "as_is_facts",
            column: "value_numeric",
            row: 0,
            reason: "'12345678901234567890123456789' has more than 38 digits".to_owned(),
        });

        let result = AsIsFactRow::to_batch(&rows);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_same_digest_bytes_when_only_the_run_columns_differ() {
        let first = sample_rows().remove(0);
        let mut second = first.clone();
        second.provenance.execution_id = Uuid::from_u128(2);
        second.provenance.loaded_at = Timestamp::UNIX_EPOCH;
        let mut first_bytes = ByteForm::new();
        first.write_row(&mut first_bytes);

        let expected_result = first_bytes;

        let mut result = ByteForm::new();
        second.write_row(&mut result);

        assert_eq!(result, expected_result);
    }
}
