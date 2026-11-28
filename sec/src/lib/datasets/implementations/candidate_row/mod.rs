//! # Candidate Row
//!
//! Provides [`CandidateRow`], one row of the dataset `candidates`: a [`Candidate`], its
//! entity bucket, and its [`RunColumns`].

use std::str::FromStr;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, BooleanArray, Date32Array, Decimal128Array, Int32Array, ListArray, RecordBatch,
    StringArray, StructArray, TimestampMillisecondArray, UInt8Array, UInt16Array,
};
use arrow::datatypes::{DataType, Field, FieldRef, Fields, Schema, SchemaRef};
use bigdecimal::BigDecimal;
use xbrl::core::candidate::{Candidate, CandidateValue};
use xbrl::core::concept::{ConceptName, NamespaceFamily};
use xbrl::core::confidence::Confidence;
use xbrl::core::elements::CanonicalElementKind;
use xbrl::core::fact::{Decimals, FactKey, FilingSource, TaxonomyVersion};
use xbrl::core::filing::{EntityId, FilingId};
use xbrl::core::invalid_value::InvalidValue;
use xbrl::core::release::{Sha256, TaxonomyRelease, TaxonomyReleaseId};
use xbrl::core::rule::RuleId;
use xbrl::core::scope::Scope;
use xbrl::core::unit::Unit;

use crate::datasets::columns::{
    BatchColumns, confidence_from_name, date_from_days, date32, decimal128_array, decimals_arrays,
    decimals_from_columns, dimensions_array, dimensions_at, dimensions_item, failed_batch,
    item_range, list_array, nullable, number_from_decimal128, period_from_columns, timestamp_array,
    timestamp_from_milliseconds, timestamp_type, unexpected_value, utf8_or_null, write_decimals,
    write_dimensions,
};
use crate::datasets::constants::{DECIMAL_PRECISION, DECIMAL_SCALE};
use crate::datasets::{ArrowRow, ArrowRowError, PartOrder, RunColumns};
use crate::shared::row_digest::{ByteForm, DigestRow};

/// One row of `candidates`: the value that one filing gives for one standard concept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateRow {
    /// The candidate of the filing.
    pub candidate: Candidate,
    /// The entity bucket of the entity, from 0 to 63. A series task reads the row groups of its
    /// buckets.
    pub entity_bucket: u8,
    /// The version of the transform output and the run that wrote the row.
    pub run: RunColumns,
}

impl ArrowRow for CandidateRow {
    const DATASET: &'static str = "candidates";
    const SCHEMA_VERSION: u16 = 1;

    fn schema(version: u16) -> Option<SchemaRef> {
        (version == 1).then(schema_v1)
    }

    fn to_batch(rows: &[Self]) -> Result<RecordBatch, ArrowRowError> {
        let candidates = || rows.iter().map(|row| &row.candidate);
        let text = |value: fn(&Candidate) -> Option<String>| -> ArrayRef {
            Arc::new(StringArray::from_iter(candidates().map(value)))
        };
        let values = decimal128_array(
            Self::DATASET,
            "value",
            candidates().map(|candidate| reported(candidate).map(|(value, ..)| value)),
        )?;
        let [decimals, decimals_infinite] = decimals_arrays(
            candidates().map(|candidate| reported(candidate).and_then(|(_, decimals, _)| decimals)),
        );
        let mut columns: Vec<ArrayRef> = vec![
            text(|candidate| Some(candidate.entity.scheme().to_owned())),
            text(|candidate| Some(candidate.entity.id().to_owned())),
            text(|candidate| Some(candidate.concept.name().to_owned())),
            Arc::new(Date32Array::from_iter_values(
                candidates().map(|candidate| date32(candidate.period.end())),
            )),
            Arc::new(Date32Array::from_iter(
                candidates().map(|candidate| candidate.period.start().map(date32)),
            )),
            Arc::new(UInt16Array::from_iter_values(
                candidates().map(|candidate| candidate.period.quarters()),
            )),
            dimensions_array(candidates().map(|candidate| &candidate.dimensions)),
            text(|candidate| Some(candidate.unit.to_string())),
            values,
            Arc::new(StringArray::from_iter(candidates().map(|candidate| {
                reported(candidate).map(|(.., confidence)| confidence.to_string())
            }))),
            text(|candidate| Some(candidate.rule.to_string())),
            text(|candidate| Some(candidate.release.id.to_string())),
            text(|candidate| Some(candidate.release.checksum.to_string())),
            inputs_array(rows)?,
            text(|candidate| candidate.scope.map(|scope| scope.to_string())),
            text(|candidate| Some(candidate.source.filing.to_string())),
            Arc::new(Date32Array::from_iter_values(
                candidates().map(|candidate| date32(candidate.source.filed)),
            )),
            timestamp_array(candidates().map(|candidate| candidate.source.accepted)),
            decimals,
            decimals_infinite,
            Arc::new(BooleanArray::from_iter(candidates().map(|candidate| {
                Some(matches!(candidate.value, CandidateValue::Blocked))
            }))),
            Arc::new(UInt8Array::from_iter_values(
                rows.iter().map(|row| row.entity_bucket),
            )),
        ];
        let runs: Vec<RunColumns> = rows.iter().map(|row| row.run).collect();
        columns.extend(RunColumns::arrays(&runs));
        RecordBatch::try_new(schema_v1(), columns)
            .map_err(|error| failed_batch(Self::DATASET, &error))
    }

    fn from_batch(batch: &RecordBatch, version: u16) -> Result<Vec<Self>, ArrowRowError> {
        let columns = BatchColumns::check::<Self>(batch, version)?;
        let runs = RunColumns::read(&columns)?;
        let reader = CandidateColumns::new(&columns)?;
        runs.into_iter()
            .enumerate()
            .map(|(row, run)| {
                Ok(Self {
                    candidate: reader.candidate(row)?,
                    entity_bucket: reader.entity_buckets.value(row),
                    run,
                })
            })
            .collect()
    }
}

/// Sorts by `entity_bucket`, then by the value key, which starts with `filing_id`. Each
/// `entity_bucket` starts a new row group.
impl PartOrder for CandidateRow {
    fn write_sort_key(&self, byte_form: &mut ByteForm) {
        self.write_row_group_key(byte_form);
        self.write_row_key(byte_form);
    }

    fn write_row_group_key(&self, byte_form: &mut ByteForm) {
        byte_form.integer(self.entity_bucket);
    }
}

impl DigestRow for CandidateRow {
    fn write_group_id(&self, byte_form: &mut ByteForm) {
        byte_form.utf8(&self.candidate.source.filing.to_string());
    }

    /// Appends `filing_id`, `concept`, `period_end`, `quarters`, `unit`, and `dimensions`.
    fn write_row_key(&self, byte_form: &mut ByteForm) {
        let candidate = &self.candidate;
        byte_form
            .utf8(&candidate.source.filing.to_string())
            .utf8(candidate.concept.name())
            .date(candidate.period.end())
            .integer(candidate.period.quarters())
            .utf8(&candidate.unit.to_string());
        write_dimensions(byte_form, &candidate.dimensions);
    }

    fn write_row(&self, byte_form: &mut ByteForm) {
        let candidate = &self.candidate;
        byte_form
            .utf8(candidate.entity.scheme())
            .utf8(candidate.entity.id())
            .utf8(candidate.concept.name())
            .date(candidate.period.end());
        match candidate.period.start() {
            Some(start) => byte_form.date(start),
            None => byte_form.null(),
        };
        byte_form.integer(candidate.period.quarters());
        write_dimensions(byte_form, &candidate.dimensions);
        byte_form.utf8(&candidate.unit.to_string());
        let decimals = if let Some((value, decimals, confidence)) = reported(candidate) {
            byte_form.decimal(value).utf8(&confidence.to_string());
            decimals
        } else {
            byte_form.null().null();
            None
        };
        byte_form
            .utf8(candidate.rule.as_str())
            .utf8(candidate.release.id.as_str())
            .utf8(&candidate.release.checksum.to_string())
            .list(candidate.inputs.len());
        for key in &candidate.inputs {
            write_fact_key(byte_form, key);
        }
        match candidate.scope {
            Some(scope) => byte_form.utf8(&scope.to_string()),
            None => byte_form.null(),
        };
        byte_form
            .utf8(&candidate.source.filing.to_string())
            .date(candidate.source.filed);
        match candidate.source.accepted {
            Some(accepted) => byte_form.timestamp(accepted),
            None => byte_form.null(),
        };
        write_decimals(byte_form, decimals);
        byte_form
            .bool(matches!(candidate.value, CandidateValue::Blocked))
            .integer(self.entity_bucket);
        self.run.write_digest(byte_form);
    }
}

/// Returns the schema of version 1.
fn schema_v1() -> SchemaRef {
    let text = |name: &str, nullable: bool| Field::new(name, DataType::Utf8, nullable);
    let date = |name: &str, nullable: bool| Field::new(name, DataType::Date32, nullable);
    let mut fields = vec![
        text("entity_scheme", false),
        text("entity_id", false),
        text("concept", false),
        date("period_end", false),
        date("period_start", true),
        Field::new("quarters", DataType::UInt16, false),
        Field::new("dimensions", DataType::List(dimensions_item()), false),
        text("unit", false),
        Field::new(
            "value",
            DataType::Decimal128(DECIMAL_PRECISION, DECIMAL_SCALE),
            true,
        ),
        text("confidence", true),
        text("rule", false),
        text("release_id", false),
        text("release_checksum", false),
        Field::new("inputs", DataType::List(input_item()), false),
        text("scope", true),
        text("filing_id", false),
        date("filed", false),
        Field::new("accepted", timestamp_type(), true),
        Field::new("decimals", DataType::Int32, true),
        Field::new("decimals_infinite", DataType::Boolean, false),
        Field::new("blocked", DataType::Boolean, false),
        Field::new("entity_bucket", DataType::UInt8, false),
    ];
    fields.extend(RunColumns::fields());
    Arc::new(Schema::new(fields))
}

/// Returns the item field of the `inputs` column: a struct of the fields of a value key.
fn input_item() -> FieldRef {
    let text = |name: &str, nullable: bool| Field::new(name, DataType::Utf8, nullable);
    let key = Fields::from(vec![
        text("filing_id", false),
        text("concept_family", false),
        text("concept_name", false),
        text("taxonomy_version", true),
        Field::new("period_end", DataType::Date32, false),
        Field::new("quarters", DataType::UInt16, false),
        text("unit", true),
        Field::new("dimensions", DataType::List(dimensions_item()), false),
        Field::new("copy_rank", DataType::UInt16, false),
    ]);
    Arc::new(Field::new_list_field(DataType::Struct(key), false))
}

/// Returns the `inputs` array: per row, the value keys of the as-is facts of the candidate.
fn inputs_array(rows: &[CandidateRow]) -> Result<ArrayRef, ArrowRowError> {
    let keys = || rows.iter().flat_map(|row| &row.candidate.inputs);
    let text = |value: fn(&FactKey) -> Option<String>| -> ArrayRef {
        Arc::new(StringArray::from_iter(keys().map(value)))
    };
    let item = input_item();
    let DataType::Struct(fields) = item.data_type() else {
        unreachable!("Given the item field of `inputs`, a struct of the fields of a value key")
    };
    let key_columns: Vec<ArrayRef> = vec![
        text(|key| Some(key.filing.to_string())),
        text(|key| Some(key.concept.family().to_string())),
        text(|key| Some(key.concept.local_name().to_owned())),
        text(|key| key.taxonomy_version.as_ref().map(ToString::to_string)),
        Arc::new(Date32Array::from_iter_values(
            keys().map(|key| date32(key.period_end)),
        )),
        Arc::new(UInt16Array::from_iter_values(
            keys().map(|key| key.quarters),
        )),
        text(|key| key.unit.as_ref().map(ToString::to_string)),
        dimensions_array(keys().map(|key| &key.dimensions)),
        Arc::new(UInt16Array::from_iter_values(
            keys().map(|key| key.copy_rank),
        )),
    ];
    let items = StructArray::try_new(Fields::clone(fields), key_columns, None)
        .map_err(|error| failed_batch(CandidateRow::DATASET, &error))?;
    let lengths = rows.iter().map(|row| row.candidate.inputs.len()).collect();
    Ok(list_array(item, lengths, Arc::new(items)))
}

/// Appends a value key as a struct of its nine fields.
fn write_fact_key(byte_form: &mut ByteForm, key: &FactKey) {
    byte_form
        .struct_(9)
        .utf8(&key.filing.to_string())
        .utf8(key.concept.family().as_str())
        .utf8(key.concept.local_name());
    utf8_or_null(
        byte_form,
        key.taxonomy_version.as_ref().map(TaxonomyVersion::as_str),
    );
    byte_form.date(key.period_end).integer(key.quarters);
    utf8_or_null(
        byte_form,
        key.unit.as_ref().map(ToString::to_string).as_deref(),
    );
    write_dimensions(byte_form, &key.dimensions);
    byte_form.integer(key.copy_rank);
}

/// Returns the value, the decimals, and the confidence of a candidate, or `None` if it is
/// blocked.
const fn reported(candidate: &Candidate) -> Option<(&BigDecimal, Option<Decimals>, Confidence)> {
    match &candidate.value {
        CandidateValue::Reported {
            value,
            decimals,
            confidence,
        } => Some((value, *decimals, *confidence)),
        CandidateValue::Blocked => None,
    }
}

/// The arrays of the candidate columns of one batch.
struct CandidateColumns<'a> {
    dataset: &'static str,
    entity_schemes: &'a StringArray,
    entity_ids: &'a StringArray,
    concepts: &'a StringArray,
    period_ends: &'a Date32Array,
    period_starts: &'a Date32Array,
    quarters: &'a UInt16Array,
    dimensions: &'a ListArray,
    units: &'a StringArray,
    values: &'a Decimal128Array,
    confidences: &'a StringArray,
    rules: &'a StringArray,
    release_ids: &'a StringArray,
    release_checksums: &'a StringArray,
    inputs: &'a ListArray,
    input_keys: FactKeyColumns<'a>,
    scopes: &'a StringArray,
    filing_ids: &'a StringArray,
    filed_dates: &'a Date32Array,
    accepted_times: &'a TimestampMillisecondArray,
    decimals: &'a Int32Array,
    decimals_infinite: &'a BooleanArray,
    blocked: &'a BooleanArray,
    entity_buckets: &'a UInt8Array,
}

impl<'a> CandidateColumns<'a> {
    /// Takes the arrays of the candidate columns from a batch that passed the schema check.
    fn new(columns: &BatchColumns<'a>) -> Result<Self, ArrowRowError> {
        Ok(Self {
            dataset: columns.dataset(),
            entity_schemes: columns.get("entity_scheme")?,
            entity_ids: columns.get("entity_id")?,
            concepts: columns.get("concept")?,
            period_ends: columns.get("period_end")?,
            period_starts: columns.get("period_start")?,
            quarters: columns.get("quarters")?,
            dimensions: columns.get("dimensions")?,
            units: columns.get("unit")?,
            values: columns.get("value")?,
            confidences: columns.get("confidence")?,
            rules: columns.get("rule")?,
            release_ids: columns.get("release_id")?,
            release_checksums: columns.get("release_checksum")?,
            inputs: columns.get("inputs")?,
            input_keys: FactKeyColumns::new(columns)?,
            scopes: columns.get("scope")?,
            filing_ids: columns.get("filing_id")?,
            filed_dates: columns.get("filed")?,
            accepted_times: columns.get("accepted")?,
            decimals: columns.get("decimals")?,
            decimals_infinite: columns.get("decimals_infinite")?,
            blocked: columns.get("blocked")?,
            entity_buckets: columns.get("entity_bucket")?,
        })
    }

    /// Reads the candidate of one row.
    fn candidate(&self, row: usize) -> Result<Candidate, ArrowRowError> {
        let invalid = |column: &'static str| {
            move |source: InvalidValue| ArrowRowError::InvalidColumnValue {
                dataset: self.dataset,
                column,
                row,
                source,
            }
        };
        let unexpected = |column: &'static str| {
            move |reason| unexpected_value(self.dataset, column, row, reason)
        };
        Ok(Candidate {
            source: FilingSource {
                filing: FilingId::from_str(self.filing_ids.value(row))
                    .map_err(invalid("filing_id"))?,
                filed: date_from_days(self.filed_dates.value(row)).map_err(unexpected("filed"))?,
                accepted: nullable(self.accepted_times, row)
                    .map(timestamp_from_milliseconds)
                    .transpose()
                    .map_err(unexpected("accepted"))?,
            },
            entity: EntityId::new(self.entity_schemes.value(row), self.entity_ids.value(row))
                .map_err(invalid("entity_id"))?,
            concept: CanonicalElementKind::from_str(self.concepts.value(row))
                .map_err(invalid("concept"))?,
            period: period_from_columns(
                self.dataset,
                row,
                self.period_ends.value(row),
                nullable(self.period_starts, row),
                self.quarters.value(row),
            )?,
            unit: Unit::from_str(self.units.value(row)).map_err(invalid("unit"))?,
            dimensions: dimensions_at(self.dimensions, row).map_err(invalid("dimensions"))?,
            value: self.value(row)?,
            rule: RuleId::from_str(self.rules.value(row)).map_err(invalid("rule"))?,
            release: TaxonomyRelease {
                id: TaxonomyReleaseId::new(self.release_ids.value(row))
                    .map_err(invalid("release_id"))?,
                checksum: Sha256::from_str(self.release_checksums.value(row))
                    .map_err(invalid("release_checksum"))?,
            },
            inputs: item_range(self.inputs, row)
                .map(|item| self.input_keys.key(self.dataset, row, item))
                .collect::<Result<_, _>>()?,
            scope: nullable(self.scopes, row)
                .map(Scope::from_str)
                .transpose()
                .map_err(invalid("scope"))?,
        })
    }

    /// Reads the value of one row from `blocked`, `value`, `confidence`, and the decimals.
    fn value(&self, row: usize) -> Result<CandidateValue, ArrowRowError> {
        let decimals = decimals_from_columns(
            self.dataset,
            row,
            nullable(self.decimals, row),
            self.decimals_infinite.value(row),
        )?;
        let unexpected =
            |column, reason: &str| unexpected_value(self.dataset, column, row, reason.to_owned());
        let value = nullable(self.values, row);
        let confidence = nullable(self.confidences, row);
        match (self.blocked.value(row), value, confidence) {
            (true, None, None) if decimals.is_none() => Ok(CandidateValue::Blocked),
            (true, ..) => Err(unexpected(
                "blocked",
                "a blocked candidate with a value, a confidence, or decimals",
            )),
            (false, Some(digits), Some(name)) => Ok(CandidateValue::Reported {
                value: number_from_decimal128(digits),
                decimals,
                confidence: confidence_from_name(name).ok_or_else(|| {
                    unexpected_value(
                        self.dataset,
                        "confidence",
                        row,
                        format!("unknown confidence '{name}'"),
                    )
                })?,
            }),
            (false, None, _) => Err(unexpected(
                "value",
                "null in a candidate that is not blocked",
            )),
            (false, Some(_), None) => Err(unexpected(
                "confidence",
                "null in a candidate that is not blocked",
            )),
        }
    }
}

/// The arrays of the fields of the value keys in the `inputs` column of one batch.
///
/// Each array holds the keys of every row. The offsets of `inputs` give the keys of one row.
struct FactKeyColumns<'a> {
    filing_ids: &'a StringArray,
    concept_families: &'a StringArray,
    concept_names: &'a StringArray,
    taxonomy_versions: &'a StringArray,
    period_ends: &'a Date32Array,
    quarters: &'a UInt16Array,
    units: &'a StringArray,
    dimensions: &'a ListArray,
    copy_ranks: &'a UInt16Array,
}

impl<'a> FactKeyColumns<'a> {
    /// Takes the arrays of the fields of `inputs` from a batch that passed the schema check.
    fn new(columns: &BatchColumns<'a>) -> Result<Self, ArrowRowError> {
        let field = |name| columns.item_field("inputs", name);
        Ok(Self {
            filing_ids: field("filing_id")?,
            concept_families: field("concept_family")?,
            concept_names: field("concept_name")?,
            taxonomy_versions: field("taxonomy_version")?,
            period_ends: columns.item_field("inputs", "period_end")?,
            quarters: columns.item_field("inputs", "quarters")?,
            units: field("unit")?,
            dimensions: columns.item_field("inputs", "dimensions")?,
            copy_ranks: columns.item_field("inputs", "copy_rank")?,
        })
    }

    /// Reads the value key at index `item` of the keys, a key of the row `row`.
    ///
    /// An error names the column `inputs` and the row.
    fn key(
        &self,
        dataset: &'static str,
        row: usize,
        item: usize,
    ) -> Result<FactKey, ArrowRowError> {
        let invalid = |source| ArrowRowError::InvalidColumnValue {
            dataset,
            column: "inputs",
            row,
            source,
        };
        Ok(FactKey {
            filing: FilingId::from_str(self.filing_ids.value(item)).map_err(invalid)?,
            concept: ConceptName::new(
                NamespaceFamily::new(self.concept_families.value(item)).map_err(invalid)?,
                self.concept_names.value(item),
            )
            .map_err(invalid)?,
            taxonomy_version: nullable(self.taxonomy_versions, item)
                .map(TaxonomyVersion::new)
                .transpose()
                .map_err(invalid)?,
            period_end: date_from_days(self.period_ends.value(item))
                .map_err(|reason| unexpected_value(dataset, "inputs", row, reason))?,
            quarters: self.quarters.value(item),
            unit: nullable(self.units, item)
                .map(Unit::from_str)
                .transpose()
                .map_err(invalid)?,
            dimensions: dimensions_at(self.dimensions, item).map_err(invalid)?,
            copy_rank: self.copy_ranks.value(item),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use arrow::array::Array;
    use bigdecimal::BigDecimal;
    use jiff::Timestamp;
    use jiff::civil::date;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;
    use xbrl::core::elements::IncomeStatementConcept;
    use xbrl::core::fact::{Decimals, Dimensions};
    use xbrl::core::period::{Duration, Period};

    use super::*;
    use crate::datasets::implementations::run_columns::sample_run_columns;
    use crate::datasets::{FOLDER_LABELS, schema_text};
    use xbrl::core::invalid_value::InvalidValueReason;

    const PARSE: &str = "Given hardcoded valid parts, the creation should always succeed";

    /// The filing `sec:0000320193-26-000001`.
    fn filing() -> FilingId {
        FilingId::from_str("sec:0000320193-26-000001").expect(PARSE)
    }

    /// A value key of `us-gaap:Revenues` in the filing, with every field set.
    fn full_key() -> FactKey {
        FactKey {
            filing: filing(),
            concept: ConceptName::new(NamespaceFamily::new("us-gaap").expect(PARSE), "Revenues")
                .expect(PARSE),
            taxonomy_version: Some(TaxonomyVersion::new("us-gaap/2026").expect(PARSE)),
            period_end: date(2026, 9, 26),
            quarters: 4,
            unit: Some(Unit::from_str("USD").expect(PARSE)),
            dimensions: Dimensions::new([("StatementBusinessSegmentsAxis", "IPhoneMember")])
                .expect(PARSE),
            copy_rank: 0,
        }
    }

    /// A value key with every nullable field empty and the copy rank 1.
    fn sparse_key() -> FactKey {
        FactKey {
            taxonomy_version: None,
            unit: None,
            dimensions: Dimensions::default(),
            copy_rank: 1,
            ..full_key()
        }
    }

    /// A candidate of revenue for the fiscal year that ends on 2026-09-26.
    fn candidate(value: CandidateValue) -> Candidate {
        Candidate {
            source: FilingSource {
                filing: filing(),
                filed: date(2026, 11, 6),
                accepted: Some(Timestamp::from_millisecond(1_793_966_700_000).expect(PARSE)),
            },
            entity: EntityId::new("http://www.sec.gov/CIK", "0000320193").expect(PARSE),
            concept: IncomeStatementConcept::Revenue.element(),
            period: Period::Duration(
                Duration::from_dates(date(2025, 9, 28), date(2026, 9, 26)).expect(PARSE),
            ),
            unit: Unit::from_str("USD").expect(PARSE),
            dimensions: Dimensions::default(),
            value,
            rule: RuleId::from_str("us-gaap/alias/revenue@1").expect(PARSE),
            release: TaxonomyRelease {
                id: TaxonomyReleaseId::new("us-gaap/2026").expect(PARSE),
                checksum: Sha256::from_str(
                    "a92083bfe2d99f604c258a3a192602c4e8c8292d08f51cd3d34db30b80353b3c",
                )
                .expect(PARSE),
            },
            inputs: vec![full_key()],
            scope: None,
        }
    }

    /// A reported value with its decimals and confidence.
    fn reported(text: &str, decimals: Option<Decimals>, confidence: Confidence) -> CandidateValue {
        CandidateValue::Reported {
            value: BigDecimal::from_str(text)
                .expect("Given a hardcoded decimal text, the parse should always succeed"),
            decimals,
            confidence,
        }
    }

    /// The fixture rows: 38 digits with two keys, a non-ASCII member with `Infinite` decimals and
    /// a scope, and a blocked instant with every nullable field empty.
    fn sample_rows() -> Vec<CandidateRow> {
        let row = |candidate, entity_bucket| CandidateRow {
            candidate,
            entity_bucket,
            run: sample_run_columns(),
        };
        let mut digits = candidate(reported(
            "1234567890123456789012345678.1234567890",
            Some(Decimals::Finite(-6)),
            Confidence::Exact,
        ));
        digits.inputs = vec![full_key(), sparse_key()];
        let mut segment = candidate(reported(
            "-6.11",
            Some(Decimals::Infinite),
            Confidence::Synonym,
        ));
        segment.dimensions =
            Dimensions::new([("StatementGeographicalAxis", "SociétéGénérale株式会社Member")])
                .expect(PARSE);
        segment.scope = Some(Scope::ExcludesRestrictedCash);
        let mut blocked = candidate(CandidateValue::Blocked);
        blocked.source.accepted = None;
        blocked.period = Period::Instant {
            end: date(2026, 9, 26),
        };
        blocked.inputs = vec![sparse_key()];
        vec![row(digits, 7), row(segment, 7), row(blocked, 63)]
    }

    /// Returns the batch of the first fixture row, with the column `name` replaced by `column`.
    fn batch_with(name: &str, column: ArrayRef) -> RecordBatch {
        row_batch_with(0, name, column)
    }

    /// Returns the batch of the blocked fixture row, with the column `name` replaced by `column`.
    fn blocked_batch_with(name: &str, column: ArrayRef) -> RecordBatch {
        row_batch_with(2, name, column)
    }

    /// Returns the batch of fixture row `row`, with the column `name` replaced by `column`.
    fn row_batch_with(row: usize, name: &str, column: ArrayRef) -> RecordBatch {
        let batch = CandidateRow::to_batch(&sample_rows()[row..=row])
            .expect("Given rows whose values fit their columns, the batch should always build");
        let index = batch
            .schema()
            .index_of(name)
            .expect("Given a column of the schema of version 1, the lookup should always succeed");
        let mut columns = batch.columns().to_vec();
        columns[index] = column;
        RecordBatch::try_new(batch.schema(), columns)
            .expect("Given a column of the same type, the batch should always build")
    }

    /// Returns the batch of the first fixture row, with the field `name` of its two value keys
    /// replaced by `field`.
    fn batch_with_input_field(name: &str, field: ArrayRef) -> RecordBatch {
        let batch = CandidateRow::to_batch(&sample_rows()[..1])
            .expect("Given rows whose values fit their columns, the batch should always build");
        let inputs = batch
            .column_by_name("inputs")
            .and_then(|column| column.as_any().downcast_ref::<ListArray>())
            .expect("Given a batch that `to_batch` built, `inputs` should always be a list");
        let keys = inputs
            .values()
            .as_any()
            .downcast_ref::<StructArray>()
            .expect("Given the `inputs` column, its items should always be structs");
        let (index, _) = keys
            .fields()
            .find(name)
            .expect("Given a field of a value key, the lookup should always succeed");
        let mut key_columns = keys.columns().to_vec();
        key_columns[index] = field;
        let keys = StructArray::try_new(keys.fields().clone(), key_columns, None)
            .expect("Given a field of the same type and length, the struct should always build");
        let inputs =
            ListArray::try_new(input_item(), inputs.offsets().clone(), Arc::new(keys), None)
                .expect("Given the offsets of the same list, the list should always build");
        batch_with("inputs", Arc::new(inputs))
    }

    /// Returns the column that the error of reading the batch names.
    fn failing_column(batch: &RecordBatch) -> Option<&'static str> {
        match CandidateRow::from_batch(batch, 1) {
            Err(
                ArrowRowError::UnexpectedColumnValue { column, .. }
                | ArrowRowError::InvalidColumnValue { column, .. },
            ) => Some(column),
            _ => None,
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_candidate_row() {
        implements_auto_traits::<CandidateRow>();
    }

    const fn implements_value_traits<T: Debug + Clone + PartialEq + Eq>() {}
    #[test]
    const fn should_implement_value_traits_for_candidate_row() {
        implements_value_traits::<CandidateRow>();
    }

    #[test]
    fn should_return_equal_rows_when_rows_pass_through_a_batch() {
        let rows = sample_rows();

        let expected_result = sample_rows();

        let batch = CandidateRow::to_batch(&rows)
            .expect("Given rows whose values fit their columns, the batch should always build");
        let result = CandidateRow::from_batch(&batch, CandidateRow::SCHEMA_VERSION)
            .expect("Given a batch that `to_batch` built, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_match_the_checked_in_snapshot_for_the_schema_of_version_1() {
        let schema = CandidateRow::schema(1).expect("Given version 1, the schema should exist");

        let expected_result = include_str!("schema_v1.txt");

        let result = schema_text(&schema);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_no_field_after_a_folder_label_for_candidates() {
        let schema = CandidateRow::schema(CandidateRow::SCHEMA_VERSION)
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
        let batch = CandidateRow::to_batch(&sample_rows())
            .expect("Given rows whose values fit their columns, the batch should always build");

        let expected_result = Err(ArrowRowError::UnknownSchemaVersion {
            dataset: "candidates",
            version: 2,
        });

        let result = CandidateRow::from_batch(&batch, 2);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_rows_when_the_batch_holds_no_rows() {
        let rows: Vec<CandidateRow> = Vec::new();

        let expected_result: Vec<CandidateRow> = Vec::new();

        let batch =
            CandidateRow::to_batch(&rows).expect("Given no rows, the batch should always build");
        let result = CandidateRow::from_batch(&batch, CandidateRow::SCHEMA_VERSION)
            .expect("Given a batch that `to_batch` built, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_value_has_more_than_10_digits_after_the_point() {
        let rows = vec![CandidateRow {
            candidate: candidate(reported("0.12345678901", None, Confidence::Exact)),
            entity_bucket: 7,
            run: sample_run_columns(),
        }];

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "candidates",
            column: "value",
            row: 0,
            reason: "'0.12345678901' has more than 10 digits after the point".to_owned(),
        });

        let result = CandidateRow::to_batch(&rows);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_blocked_row_has_a_value() {
        let batch = batch_with("blocked", Arc::new(BooleanArray::from(vec![true])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "candidates",
            column: "blocked",
            row: 0,
            reason: "a blocked candidate with a value, a confidence, or decimals".to_owned(),
        });

        let result = CandidateRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_row_that_is_not_blocked_has_no_confidence() {
        let batch = batch_with(
            "confidence",
            Arc::new(StringArray::from(vec![None::<&str>])),
        );

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "candidates",
            column: "confidence",
            row: 0,
            reason: "null in a candidate that is not blocked".to_owned(),
        });

        let result = CandidateRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_row_that_is_not_blocked_has_no_value() {
        let batch = batch_with(
            "value",
            Arc::new(
                Decimal128Array::from(vec![None::<i128>])
                    .with_precision_and_scale(DECIMAL_PRECISION, DECIMAL_SCALE)
                    .expect("Given the precision and scale of the column, the array should always build"),
            ),
        );

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "candidates",
            column: "value",
            row: 0,
            reason: "null in a candidate that is not blocked".to_owned(),
        });

        let result = CandidateRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_blocked_row_has_only_decimals() {
        let batch = blocked_batch_with("decimals", Arc::new(Int32Array::from(vec![-6])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "candidates",
            column: "blocked",
            row: 0,
            reason: "a blocked candidate with a value, a confidence, or decimals".to_owned(),
        });

        let result = CandidateRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_blocked_row_has_only_a_confidence() {
        let batch = blocked_batch_with("confidence", Arc::new(StringArray::from(vec!["Exact"])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "candidates",
            column: "blocked",
            row: 0,
            reason: "a blocked candidate with a value, a confidence, or decimals".to_owned(),
        });

        let result = CandidateRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_confidence_is_unknown() {
        let batch = batch_with("confidence", Arc::new(StringArray::from(vec!["Certain"])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "candidates",
            column: "confidence",
            row: 0,
            reason: "unknown confidence 'Certain'".to_owned(),
        });

        let result = CandidateRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_the_scope_when_a_row_has_a_scope() {
        let batch = batch_with(
            "scope",
            Arc::new(StringArray::from(vec!["ExcludesRestrictedCash"])),
        );

        let expected_result = vec![Some(Scope::ExcludesRestrictedCash)];

        let result: Vec<Option<Scope>> = CandidateRow::from_batch(&batch, 1)
            .expect("Given a batch with a known scope, the read should always succeed")
            .into_iter()
            .map(|row| row.candidate.scope)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_scope_is_unknown() {
        let batch = batch_with(
            "scope",
            Arc::new(StringArray::from(vec!["IncludesRestrictedCash"])),
        );

        let expected_result = Err(ArrowRowError::InvalidColumnValue {
            dataset: "candidates",
            column: "scope",
            row: 0,
            source: InvalidValue::new(
                "Scope",
                InvalidValueReason::NoMatchingName("scope"),
                "IncludesRestrictedCash",
            ),
        });

        let result = CandidateRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_row_has_finite_decimals_and_decimals_infinite() {
        let batch = batch_with(
            "decimals_infinite",
            Arc::new(BooleanArray::from(vec![true])),
        );

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "candidates",
            column: "decimals",
            row: 0,
            reason: "a finite value with 'decimals_infinite' true".to_owned(),
        });

        let result = CandidateRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_quarters_differ_from_the_period_dates() {
        let batch = batch_with("quarters", Arc::new(UInt16Array::from(vec![5])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "candidates",
            column: "quarters",
            row: 0,
            reason: "the dates give '4' quarters".to_owned(),
        });

        let result = CandidateRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_concept_is_unknown() {
        let batch = batch_with("concept", Arc::new(StringArray::from(vec!["Revenues"])));

        let expected_result = Some("concept");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_rule_breaks_its_form() {
        let batch = batch_with("rule", Arc::new(StringArray::from(vec!["revenue"])));

        let expected_result = Some("rule");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_release_id_breaks_its_form() {
        let batch = batch_with("release_id", Arc::new(StringArray::from(vec![""])));

        let expected_result = Some("release_id");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_release_checksum_is_not_hex() {
        let batch = batch_with("release_checksum", Arc::new(StringArray::from(vec!["abc"])));

        let expected_result = Some("release_checksum");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_filing_id_breaks_its_form() {
        let batch = batch_with("filing_id", Arc::new(StringArray::from(vec!["0000320193"])));

        let expected_result = Some("filing_id");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_unit_is_empty() {
        let batch = batch_with("unit", Arc::new(StringArray::from(vec![""])));

        let expected_result = Some("unit");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_inputs_when_the_filing_id_of_a_value_key_breaks_its_form() {
        let batch = batch_with_input_field(
            "filing_id",
            Arc::new(StringArray::from(vec!["sec:0000320193-26-000001", "x"])),
        );

        let expected_result = Some("inputs");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_inputs_when_the_period_end_of_a_value_key_lies_past_the_dates_of_jiff() {
        let batch = batch_with_input_field(
            "period_end",
            Arc::new(Date32Array::from(vec![20_722, i32::MAX])),
        );

        let expected_result = Some("inputs");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_transform_version_is_not_a_number() {
        let batch = batch_with("transform_version", Arc::new(StringArray::from(vec!["v1"])));

        let expected_result = Some("transform_version");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_same_digest_bytes_when_only_the_run_columns_differ() {
        let first = sample_rows().remove(0);
        let mut second = first.clone();
        second.run.execution_id = Uuid::from_u128(2);
        second.run.loaded_at = Timestamp::UNIX_EPOCH;
        let mut first_bytes = ByteForm::new();
        first.write_row(&mut first_bytes);

        let expected_result = first_bytes;

        let mut result = ByteForm::new();
        second.write_row(&mut result);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_filing_id_as_the_digest_group() {
        let row = sample_rows().remove(2);

        // marker 01 | length 00000018 | text sec:0000320193-26-000001
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x18sec:0000320193-26-000001";

        let mut byte_form = ByteForm::new();
        row.write_group_id(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_filing_the_concept_the_period_the_unit_and_the_dimensions_as_the_row_key() {
        let row = sample_rows().remove(2);

        // filing_id | concept | period_end | quarters 0 | unit | dimensions empty list
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x18sec:0000320193-26-000001\
\x01\x00\x00\x00\x07Revenue\
\x01\x00\x00\x00\x0a2026-09-26\
\x01\x00\x00\x00\x010\
\x01\x00\x00\x00\x03USD\
\x01\x00\x00\x00\x00";

        let mut byte_form = ByteForm::new();
        row.write_row_key(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_every_column_but_the_run_ids_in_schema_order_as_the_row() {
        let row = sample_rows().remove(2);

        // entity_scheme | entity_id | concept | period_end | period_start null | quarters 0 |
        // dimensions empty | unit | value null | confidence null | rule | release_id |
        // release_checksum | inputs: list of 1, struct of 9 (filing_id, concept_family,
        // concept_name, taxonomy_version null, period_end, quarters 4, unit null, dimensions
        // empty, copy_rank 1) | scope null | filing_id | filed | accepted null | decimals null |
        // decimals_infinite false | blocked true | entity_bucket 63 | transform_version 1
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x16http://www.sec.gov/CIK\
\x01\x00\x00\x00\x0a0000320193\
\x01\x00\x00\x00\x07Revenue\
\x01\x00\x00\x00\x0a2026-09-26\
\x00\
\x01\x00\x00\x00\x010\
\x01\x00\x00\x00\x00\
\x01\x00\x00\x00\x03USD\
\x00\
\x00\
\x01\x00\x00\x00\x17us-gaap/alias/revenue@1\
\x01\x00\x00\x00\x0cus-gaap/2026\
\x01\x00\x00\x00\x40a92083bfe2d99f604c258a3a192602c4e8c8292d08f51cd3d34db30b80353b3c\
\x01\x00\x00\x00\x01\
\x01\x00\x00\x00\x09\
\x01\x00\x00\x00\x18sec:0000320193-26-000001\
\x01\x00\x00\x00\x07us-gaap\
\x01\x00\x00\x00\x08Revenues\
\x00\
\x01\x00\x00\x00\x0a2026-09-26\
\x01\x00\x00\x00\x014\
\x00\
\x01\x00\x00\x00\x00\
\x01\x00\x00\x00\x011\
\x00\
\x01\x00\x00\x00\x18sec:0000320193-26-000001\
\x01\x00\x00\x00\x0a2026-11-06\
\x00\
\x00\
\x01\x00\x00\x00\x05false\
\x01\x00\x00\x00\x04true\
\x01\x00\x00\x00\x0263\
\x01\x00\x00\x00\x011";

        let mut byte_form = ByteForm::new();
        row.write_row(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_value_the_confidence_and_infinite_decimals_when_a_row_is_reported() {
        let row = sample_rows().remove(1);

        // As the blocked row, but with period_start 2025-09-28, quarters 4, one dimension pair
        // with a non-ASCII member, value -6.11, confidence Synonym, a value key with every field
        // set, scope ExcludesRestrictedCash, accepted, decimals null, decimals_infinite true,
        // blocked false, entity_bucket 7
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x16http://www.sec.gov/CIK\
\x01\x00\x00\x00\x0a0000320193\
\x01\x00\x00\x00\x07Revenue\
\x01\x00\x00\x00\x0a2026-09-26\
\x01\x00\x00\x00\x0a2025-09-28\
\x01\x00\x00\x00\x014\
\x01\x00\x00\x00\x01\x01\x00\x00\x00\x02\x01\x00\x00\x00\x19StatementGeographicalAxis\x01\x00\x00\x00\x25Soci\xc3\xa9t\xc3\xa9G\xc3\xa9n\xc3\xa9rale\xe6\xa0\xaa\xe5\xbc\x8f\xe4\xbc\x9a\xe7\xa4\xbeMember\
\x01\x00\x00\x00\x03USD\
\x01\x00\x00\x00\x05-6.11\
\x01\x00\x00\x00\x07Synonym\
\x01\x00\x00\x00\x17us-gaap/alias/revenue@1\
\x01\x00\x00\x00\x0cus-gaap/2026\
\x01\x00\x00\x00\x40a92083bfe2d99f604c258a3a192602c4e8c8292d08f51cd3d34db30b80353b3c\
\x01\x00\x00\x00\x01\x01\x00\x00\x00\x09\x01\x00\x00\x00\x18sec:0000320193-26-000001\x01\x00\x00\x00\x07us-gaap\x01\x00\x00\x00\x08Revenues\x01\x00\x00\x00\x0cus-gaap/2026\x01\x00\x00\x00\x0a2026-09-26\x01\x00\x00\x00\x014\x01\x00\x00\x00\x03USD\x01\x00\x00\x00\x01\x01\x00\x00\x00\x02\x01\x00\x00\x00\x1dStatementBusinessSegmentsAxis\x01\x00\x00\x00\x0cIPhoneMember\x01\x00\x00\x00\x010\
\x01\x00\x00\x00\x16ExcludesRestrictedCash\
\x01\x00\x00\x00\x18sec:0000320193-26-000001\
\x01\x00\x00\x00\x0a2026-11-06\
\x01\x00\x00\x00\x182026-11-06T12:05:00.000Z\
\x00\
\x01\x00\x00\x00\x04true\
\x01\x00\x00\x00\x05false\
\x01\x00\x00\x00\x017\
\x01\x00\x00\x00\x011";

        let mut byte_form = ByteForm::new();
        row.write_row(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_entity_bucket_as_the_row_group_key_for_a_candidate_row() {
        let row = CandidateRow {
            candidate: candidate(reported("1", None, Confidence::Exact)),
            entity_bucket: 7,
            run: sample_run_columns(),
        };

        // marker 01 | length 00000001 | text 7
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x017";

        let mut byte_form = ByteForm::new();
        row.write_row_group_key(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }
}
