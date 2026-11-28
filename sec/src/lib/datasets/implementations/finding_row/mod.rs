//! # Finding Row
//!
//! Provides [`FindingRow`], one row of the silver dataset `findings`, with its [`FindingKind`],
//! its [`FindingPeriod`] values, and its [`FindingStep`].

use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, Date32Array, ListArray, RecordBatch, StringArray, StructArray,
    TimestampMillisecondArray, UInt16Array,
};
use arrow::datatypes::{DataType, Field, FieldRef, Fields, Schema, SchemaRef};
use jiff::Timestamp;
use jiff::civil::Date;
use xbrl::core::confidence::Confidence;
use xbrl::core::filing::{EntityId, FilingId};
use xbrl::core::invalid_value::InvalidValue;

use crate::datasets::columns::{
    BatchColumns, confidence_from_name, date_from_days, date32, failed_batch, item_range,
    list_array, list_item, list_type, nullable, string_list, timestamp_array,
    timestamp_from_milliseconds, timestamp_type, unexpected_value, utf8_or_null,
};
use crate::datasets::{ArrowRow, ArrowRowError, RunColumns};
use crate::shared::row_digest::{ByteForm, DigestRow};

/// One row of `findings`: a finding of the parser, the standardize filing step, or the series
/// step.
///
/// `sec/design/pipeline/silver_layer.md`, `findings`, names the columns. A row holds the columns,
/// not an `xbrl` finding, because the columns hold less than a finding.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FindingRow {
    /// The kind of the finding.
    pub kind: FindingKind,
    /// The entity the finding is about.
    pub entity: EntityId,
    /// The concepts of the finding, in the order that `silver_layer.md` gives for its kind.
    pub concepts: Vec<String>,
    /// The periods of the finding, sorted by `period_end`, then `quarters`.
    ///
    /// The order enters the key of the finding. [`FindingRow::to_batch`] rejects another order.
    pub periods: Vec<FindingPeriod>,
    /// The family or the row of a parser finding. `None` for the other kinds.
    pub subject: Option<String>,
    /// The cause of the finding, for example `FiscalYearChange`.
    pub reason: Option<String>,
    /// The filing of the finding, for the kinds that have one.
    pub filing: Option<FilingId>,
    /// The newest other filing that reports the same value, for `SuspiciousPeriod`.
    pub confirmed_by: Option<FilingId>,
    /// The weakest confidence of the inputs of a check.
    pub confidence: Option<Confidence>,
    /// The step that wrote the row, with the change and the run time of a series row.
    pub step: FindingStep,
    /// The version of the silver output and the run that wrote the row.
    pub run: RunColumns,
}

/// The kind of a finding: the text of the `kind` column.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FindingKind {
    /// A fact of a namespace family that no plug-in declares.
    UndeclaredFamily,
    /// A source row that breaks a rule of a core type.
    UnmappedRow,
    /// Copies of one value key with different values.
    Conflict,
    /// A period without a value in a series.
    Gap,
    /// Two series that cover the same time.
    Overlap,
    /// A later filing that reports another value.
    Restatement,
    /// A value whose period does not fit its filing.
    SuspiciousPeriod,
    /// A group of the roll-up tree whose members do not add up to the parent.
    RollUpMismatch,
    /// A group whose derived "other" member is negative.
    NegativeOther,
    /// A group whose members start on different days.
    MixedStarts,
    /// A group whose members have different scopes.
    MixedScopes,
    /// A balance whose change does not match the change of the period.
    RollForwardMismatch,
    /// A sum whose parts do not add up to its total.
    SumMismatch,
    /// A required standard concept without a value at the period of report.
    MissingRequired,
}

impl FindingKind {
    /// Every kind, in the order of the `kind` column in `silver_layer.md`.
    const ALL: [Self; 14] = [
        Self::UndeclaredFamily,
        Self::UnmappedRow,
        Self::Conflict,
        Self::Gap,
        Self::Overlap,
        Self::Restatement,
        Self::SuspiciousPeriod,
        Self::RollUpMismatch,
        Self::NegativeOther,
        Self::MixedStarts,
        Self::MixedScopes,
        Self::RollForwardMismatch,
        Self::SumMismatch,
        Self::MissingRequired,
    ];

    /// Returns the text of the `kind` column, the name of the variant.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UndeclaredFamily => "UndeclaredFamily",
            Self::UnmappedRow => "UnmappedRow",
            Self::Conflict => "Conflict",
            Self::Gap => "Gap",
            Self::Overlap => "Overlap",
            Self::Restatement => "Restatement",
            Self::SuspiciousPeriod => "SuspiciousPeriod",
            Self::RollUpMismatch => "RollUpMismatch",
            Self::NegativeOther => "NegativeOther",
            Self::MixedStarts => "MixedStarts",
            Self::MixedScopes => "MixedScopes",
            Self::RollForwardMismatch => "RollForwardMismatch",
            Self::SumMismatch => "SumMismatch",
            Self::MissingRequired => "MissingRequired",
        }
    }

    /// Returns the kind whose text is `text`, or `None` if no kind has it.
    #[must_use]
    pub fn from_name(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == text)
    }
}

impl fmt::Display for FindingKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One period of a finding: an item of the `periods` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FindingPeriod {
    /// The exact end of the period.
    pub period_end: Date,
    /// The quarters of the period, 0 for an instant.
    pub quarters: u16,
}

/// The step that wrote a finding row: the `step`, `change`, and `computed_at` columns.
///
/// A filing row has no change and no run time. A series row has both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FindingStep {
    /// A row of the parse filing step or the standardize filing step.
    Filing,
    /// A row of the series step.
    Series {
        /// The change of the finding against its open row.
        change: FindingChange,
        /// The start time of the series run.
        computed_at: Timestamp,
    },
}

impl FindingStep {
    /// Returns the text of the `step` column: `filing` or `series`.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Filing => "filing",
            Self::Series { .. } => "series",
        }
    }

    /// Returns the change of a series row, or `None` for a filing row.
    #[must_use]
    pub const fn change(&self) -> Option<FindingChange> {
        match self {
            Self::Filing => None,
            Self::Series { change, .. } => Some(*change),
        }
    }

    /// Returns the run time of a series row, or `None` for a filing row.
    #[must_use]
    pub const fn computed_at(&self) -> Option<Timestamp> {
        match self {
            Self::Filing => None,
            Self::Series { computed_at, .. } => Some(*computed_at),
        }
    }
}

/// The change of a series row of a finding: the text of the `change` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FindingChange {
    /// The finding opens.
    New,
    /// The reason, the confirming filing, or the confidence of an open finding changes.
    Changed,
    /// The finding closes.
    Resolved,
}

impl FindingChange {
    /// Every change.
    const ALL: [Self; 3] = [Self::New, Self::Changed, Self::Resolved];

    /// Returns the text of the `change` column: `new`, `changed`, or `resolved`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Changed => "changed",
            Self::Resolved => "resolved",
        }
    }
}

impl ArrowRow for FindingRow {
    const DATASET: &'static str = "findings";
    const SCHEMA_VERSION: u16 = 1;

    fn schema(version: u16) -> Option<SchemaRef> {
        (version == 1).then(schema_v1)
    }

    fn to_batch(rows: &[Self]) -> Result<RecordBatch, ArrowRowError> {
        if let Some(row) = rows
            .iter()
            .position(|row| row.periods.windows(2).any(|pair| pair[0] > pair[1]))
        {
            return Err(unexpected_value(
                Self::DATASET,
                "periods",
                row,
                "not sorted by 'period_end', then 'quarters'".to_owned(),
            ));
        }
        let schema = schema_v1();
        let text = |value: fn(&Self) -> Option<String>| -> ArrayRef {
            Arc::new(StringArray::from_iter(rows.iter().map(value)))
        };
        let concepts = rows.iter().flat_map(|row| row.concepts.iter());
        let mut columns: Vec<ArrayRef> = vec![
            text(|row| Some(row.kind.to_string())),
            text(|row| Some(row.entity.scheme().to_owned())),
            text(|row| Some(row.entity.id().to_owned())),
            list_array(
                list_item(&schema, "concepts"),
                rows.iter().map(|row| row.concepts.len()).collect(),
                Arc::new(StringArray::from_iter_values(concepts)),
            ),
            periods_array(rows)?,
            text(|row| row.subject.clone()),
            text(|row| row.reason.clone()),
            text(|row| row.filing.as_ref().map(ToString::to_string)),
            text(|row| row.confirmed_by.as_ref().map(ToString::to_string)),
            text(|row| row.confidence.map(|confidence| confidence.to_string())),
            text(|row| Some(row.step.as_str().to_owned())),
            text(|row| row.step.change().map(|change| change.as_str().to_owned())),
            timestamp_array(rows.iter().map(|row| row.step.computed_at())),
        ];
        let runs: Vec<RunColumns> = rows.iter().map(|row| row.run).collect();
        columns.extend(RunColumns::arrays(&runs));
        RecordBatch::try_new(schema, columns).map_err(|error| failed_batch(Self::DATASET, &error))
    }

    fn from_batch(batch: &RecordBatch, version: u16) -> Result<Vec<Self>, ArrowRowError> {
        let columns = BatchColumns::check::<Self>(batch, version)?;
        let runs = RunColumns::read(&columns)?;
        let reader = FindingColumns::new(&columns)?;
        runs.into_iter()
            .enumerate()
            .map(|(row, run)| reader.row(row, run))
            .collect()
    }
}

impl DigestRow for FindingRow {
    /// Appends `filing_id`, or `entity_scheme` then `entity_id` for a row without a filing.
    fn write_group_id(&self, byte_form: &mut ByteForm) {
        match &self.filing {
            Some(filing) => byte_form.utf8(&filing.to_string()),
            None => byte_form.utf8(self.entity.scheme()).utf8(self.entity.id()),
        };
    }

    /// Appends the key of a finding: `kind`, the entity, `concepts`, `periods`, `filing_id`, and
    /// `subject`.
    fn write_row_key(&self, byte_form: &mut ByteForm) {
        byte_form
            .utf8(self.kind.as_str())
            .utf8(self.entity.scheme())
            .utf8(self.entity.id());
        write_concepts(byte_form, &self.concepts);
        write_periods(byte_form, &self.periods);
        utf8_or_null(
            byte_form,
            self.filing.as_ref().map(ToString::to_string).as_deref(),
        );
        utf8_or_null(byte_form, self.subject.as_deref());
    }

    /// Appends every column in schema order, without `computed_at`, `execution_id`, and
    /// `loaded_at`.
    fn write_row(&self, byte_form: &mut ByteForm) {
        byte_form
            .utf8(self.kind.as_str())
            .utf8(self.entity.scheme())
            .utf8(self.entity.id());
        write_concepts(byte_form, &self.concepts);
        write_periods(byte_form, &self.periods);
        utf8_or_null(byte_form, self.subject.as_deref());
        utf8_or_null(byte_form, self.reason.as_deref());
        utf8_or_null(
            byte_form,
            self.filing.as_ref().map(ToString::to_string).as_deref(),
        );
        utf8_or_null(
            byte_form,
            self.confirmed_by
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
        );
        utf8_or_null(
            byte_form,
            self.confidence
                .map(|confidence| confidence.to_string())
                .as_deref(),
        );
        byte_form.utf8(self.step.as_str());
        utf8_or_null(byte_form, self.step.change().map(FindingChange::as_str));
        self.run.write_digest(byte_form);
    }
}

/// Returns the schema of version 1.
fn schema_v1() -> SchemaRef {
    let text = |name: &str, nullable: bool| Field::new(name, DataType::Utf8, nullable);
    let mut fields = vec![
        text("kind", false),
        text("entity_scheme", false),
        text("entity_id", false),
        Field::new("concepts", list_type(DataType::Utf8), false),
        Field::new("periods", DataType::List(period_item()), false),
        text("subject", true),
        text("reason", true),
        text("filing_id", true),
        text("confirmed_by", true),
        text("confidence", true),
        text("step", false),
        text("change", true),
        Field::new("computed_at", timestamp_type(), true),
    ];
    fields.extend(RunColumns::fields());
    Arc::new(Schema::new(fields))
}

/// Returns the item field of the `periods` column: a struct of `period_end` and `quarters`.
fn period_item() -> FieldRef {
    let period = Fields::from(vec![
        Field::new("period_end", DataType::Date32, false),
        Field::new("quarters", DataType::UInt16, false),
    ]);
    Arc::new(Field::new_list_field(DataType::Struct(period), false))
}

/// Returns the `periods` array: per row, the periods of the finding.
fn periods_array(rows: &[FindingRow]) -> Result<ArrayRef, ArrowRowError> {
    let periods = || rows.iter().flat_map(|row| row.periods.iter());
    let item = period_item();
    let DataType::Struct(fields) = item.data_type() else {
        unreachable!("Given the item field of `periods`, a struct of the end and the quarters")
    };
    let period_columns: Vec<ArrayRef> = vec![
        Arc::new(Date32Array::from_iter_values(
            periods().map(|period| date32(period.period_end)),
        )),
        Arc::new(UInt16Array::from_iter_values(
            periods().map(|period| period.quarters),
        )),
    ];
    let items = StructArray::try_new(Fields::clone(fields), period_columns, None)
        .map_err(|error| failed_batch(FindingRow::DATASET, &error))?;
    let lengths = rows.iter().map(|row| row.periods.len()).collect();
    Ok(list_array(item, lengths, Arc::new(items)))
}

/// Appends the concepts as a list of texts.
fn write_concepts(byte_form: &mut ByteForm, concepts: &[String]) {
    byte_form.list(concepts.len());
    for concept in concepts {
        byte_form.utf8(concept);
    }
}

/// Appends the periods as a list of structs of the end and the quarters.
fn write_periods(byte_form: &mut ByteForm, periods: &[FindingPeriod]) {
    byte_form.list(periods.len());
    for period in periods {
        byte_form
            .struct_(2)
            .date(period.period_end)
            .integer(period.quarters);
    }
}

/// The arrays of the finding columns of one batch.
struct FindingColumns<'a> {
    dataset: &'static str,
    kinds: &'a StringArray,
    entity_schemes: &'a StringArray,
    entity_ids: &'a StringArray,
    concepts: &'a ListArray,
    periods: &'a ListArray,
    period_ends: &'a Date32Array,
    period_quarters: &'a UInt16Array,
    subjects: &'a StringArray,
    reasons: &'a StringArray,
    filing_ids: &'a StringArray,
    confirmed_by: &'a StringArray,
    confidences: &'a StringArray,
    steps: &'a StringArray,
    changes: &'a StringArray,
    computed_at: &'a TimestampMillisecondArray,
}

impl<'a> FindingColumns<'a> {
    /// Takes the arrays of the finding columns from a batch that passed the schema check.
    fn new(columns: &BatchColumns<'a>) -> Result<Self, ArrowRowError> {
        Ok(Self {
            dataset: columns.dataset(),
            kinds: columns.get("kind")?,
            entity_schemes: columns.get("entity_scheme")?,
            entity_ids: columns.get("entity_id")?,
            concepts: columns.get("concepts")?,
            periods: columns.get("periods")?,
            period_ends: columns.item_field("periods", "period_end")?,
            period_quarters: columns.item_field("periods", "quarters")?,
            subjects: columns.get("subject")?,
            reasons: columns.get("reason")?,
            filing_ids: columns.get("filing_id")?,
            confirmed_by: columns.get("confirmed_by")?,
            confidences: columns.get("confidence")?,
            steps: columns.get("step")?,
            changes: columns.get("change")?,
            computed_at: columns.get("computed_at")?,
        })
    }

    /// Reads the finding of one row, with the run columns of the row.
    fn row(&self, row: usize, run: RunColumns) -> Result<FindingRow, ArrowRowError> {
        let invalid = |column: &'static str| {
            move |source: InvalidValue| ArrowRowError::InvalidColumnValue {
                dataset: self.dataset,
                column,
                row,
                source,
            }
        };
        let unexpected = |column: &'static str, reason: String| {
            unexpected_value(self.dataset, column, row, reason)
        };
        let kind = self.kinds.value(row);
        let filing = |array, column| {
            nullable(array, row)
                .map(FilingId::from_str)
                .transpose()
                .map_err(invalid(column))
        };
        Ok(FindingRow {
            kind: FindingKind::from_name(kind)
                .ok_or_else(|| unexpected("kind", format!("unknown kind '{kind}'")))?,
            entity: EntityId::new(self.entity_schemes.value(row), self.entity_ids.value(row))
                .map_err(invalid("entity_id"))?,
            concepts: string_list(self.concepts, row),
            periods: self.periods(row)?,
            subject: nullable(self.subjects, row).map(str::to_owned),
            reason: nullable(self.reasons, row).map(str::to_owned),
            filing: filing(self.filing_ids, "filing_id")?,
            confirmed_by: filing(self.confirmed_by, "confirmed_by")?,
            confidence: nullable(self.confidences, row)
                .map(|name| {
                    confidence_from_name(name).ok_or_else(|| {
                        unexpected("confidence", format!("unknown confidence '{name}'"))
                    })
                })
                .transpose()?,
            step: self.step(row)?,
            run,
        })
    }

    /// Reads the periods of one row from the items of `periods` that its offsets give.
    fn periods(&self, row: usize) -> Result<Vec<FindingPeriod>, ArrowRowError> {
        item_range(self.periods, row)
            .map(|item| {
                Ok(FindingPeriod {
                    period_end: date_from_days(self.period_ends.value(item))
                        .map_err(|reason| unexpected_value(self.dataset, "periods", row, reason))?,
                    quarters: self.period_quarters.value(item),
                })
            })
            .collect()
    }

    /// Reads the step of one row from `step`, `change`, and `computed_at`.
    fn step(&self, row: usize) -> Result<FindingStep, ArrowRowError> {
        let unexpected = |column: &'static str, reason: String| {
            unexpected_value(self.dataset, column, row, reason)
        };
        let step = self.steps.value(row);
        let change = nullable(self.changes, row);
        let computed_at = nullable(self.computed_at, row);
        match (step, change, computed_at) {
            ("filing", None, None) => Ok(FindingStep::Filing),
            ("filing", Some(_), _) => {
                Err(unexpected("change", "a value in a filing row".to_owned()))
            }
            ("filing", None, Some(_)) => Err(unexpected(
                "computed_at",
                "a value in a filing row".to_owned(),
            )),
            ("series", Some(change), Some(milliseconds)) => Ok(FindingStep::Series {
                change: FindingChange::ALL
                    .into_iter()
                    .find(|candidate| candidate.as_str() == change)
                    .ok_or_else(|| unexpected("change", format!("unknown change '{change}'")))?,
                computed_at: timestamp_from_milliseconds(milliseconds)
                    .map_err(|reason| unexpected("computed_at", reason))?,
            }),
            ("series", None, _) => Err(unexpected("change", "null in a series row".to_owned())),
            ("series", Some(_), None) => {
                Err(unexpected("computed_at", "null in a series row".to_owned()))
            }
            _ => Err(unexpected("step", format!("unknown step '{step}'"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use arrow::array::Array;
    use jiff::civil::date;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::datasets::implementations::run_columns::sample_run_columns;
    use crate::datasets::{FOLDER_LABELS, UTC_TIME_ZONE, schema_text};

    const PARSE: &str = "Given hardcoded valid parts, the creation should always succeed";

    /// The filing with the accession number `0000320193-<year>-<sequence>`.
    fn filing(accession: &str) -> FilingId {
        FilingId::from_str(&format!("sec:{accession}")).expect(PARSE)
    }

    /// A start time of a series run.
    fn computed_at() -> Timestamp {
        Timestamp::from_millisecond(1_793_966_700_000).expect(PARSE)
    }

    /// A finding about the entity `0000320193` with no optional column set.
    fn finding(kind: FindingKind, step: FindingStep) -> FindingRow {
        FindingRow {
            kind,
            entity: EntityId::new("http://www.sec.gov/CIK", "0000320193").expect(PARSE),
            concepts: Vec::new(),
            periods: Vec::new(),
            subject: None,
            reason: None,
            filing: None,
            confirmed_by: None,
            confidence: None,
            step,
            run: sample_run_columns(),
        }
    }

    /// The fixture rows: a parser finding with non-ASCII text, a series row with two periods and a
    /// confirming filing, a series row of an entity without a filing, and a filing row with every
    /// nullable column empty.
    ///
    /// No step writes the last row, because each kind sets at least one nullable column. It tests
    /// the empty columns of the schema.
    fn sample_rows() -> Vec<FindingRow> {
        let mut unmapped = finding(FindingKind::UnmappedRow, FindingStep::Filing);
        unmapped.subject = Some("us-gaap:Revenues c-1 Société Générale — 株式会社 ✓".to_owned());
        unmapped.reason = Some("NotANumber".to_owned());
        unmapped.filing = Some(filing("0000320193-26-000001"));
        let mut suspicious = finding(
            FindingKind::SuspiciousPeriod,
            FindingStep::Series {
                change: FindingChange::New,
                computed_at: computed_at(),
            },
        );
        suspicious.concepts = vec!["us-gaap:Revenues".to_owned(), "Revenue".to_owned()];
        suspicious.periods = vec![
            FindingPeriod {
                period_end: date(2025, 9, 27),
                quarters: 4,
            },
            FindingPeriod {
                period_end: date(2026, 9, 26),
                quarters: 4,
            },
        ];
        suspicious.reason = Some("StartBeforeEarliestYear,EndFarFromReport".to_owned());
        suspicious.filing = Some(filing("0000320193-26-000001"));
        suspicious.confirmed_by = Some(filing("0000320193-25-000079"));
        let mut roll_up = finding(
            FindingKind::RollUpMismatch,
            FindingStep::Series {
                change: FindingChange::Changed,
                computed_at: computed_at(),
            },
        );
        roll_up.concepts = vec!["Liabilities".to_owned(), "LiabilitiesCurrent".to_owned()];
        roll_up.periods = vec![FindingPeriod {
            period_end: date(2026, 9, 26),
            quarters: 0,
        }];
        roll_up.reason = Some("liabilities".to_owned());
        roll_up.confidence = Some(Confidence::Synonym);
        let gap = finding(FindingKind::Gap, FindingStep::Filing);
        vec![unmapped, suspicious, roll_up, gap]
    }

    /// Returns the batch of fixture row `row`, with the column `name` replaced by `column`.
    fn batch_with(row: usize, name: &str, column: ArrayRef) -> RecordBatch {
        let batch = FindingRow::to_batch(&sample_rows()[row..=row])
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

    /// Returns the batch of the series fixture row, with the `period_end` of its two periods
    /// replaced by `period_ends`.
    fn batch_with_period_ends(period_ends: Date32Array) -> RecordBatch {
        let batch = FindingRow::to_batch(&sample_rows()[1..=1])
            .expect("Given rows whose values fit their columns, the batch should always build");
        let periods = batch
            .column_by_name("periods")
            .and_then(|column| column.as_any().downcast_ref::<ListArray>())
            .expect("Given a batch that `to_batch` built, `periods` should always be a list");
        let items = periods
            .values()
            .as_any()
            .downcast_ref::<StructArray>()
            .expect("Given the `periods` column, its items should always be structs");
        let mut item_columns = items.columns().to_vec();
        item_columns[0] = Arc::new(period_ends);
        let items = StructArray::try_new(items.fields().clone(), item_columns, None)
            .expect("Given a field of the same type and length, the struct should always build");
        let periods = ListArray::try_new(
            period_item(),
            periods.offsets().clone(),
            Arc::new(items),
            None,
        )
        .expect("Given the offsets of the same list, the list should always build");
        batch_with(1, "periods", Arc::new(periods))
    }

    /// Returns the column that the error of reading the batch names.
    fn failing_column(batch: &RecordBatch) -> Option<&'static str> {
        match FindingRow::from_batch(batch, 1) {
            Err(
                ArrowRowError::UnexpectedColumnValue { column, .. }
                | ArrowRowError::InvalidColumnValue { column, .. },
            ) => Some(column),
            _ => None,
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_finding_row() {
        implements_auto_traits::<FindingRow>();
    }

    const fn implements_value_traits<T: Debug + Clone + PartialEq + Eq + Hash>() {}
    #[test]
    const fn should_implement_value_traits_for_finding_row() {
        implements_value_traits::<FindingRow>();
    }

    const fn implements_ordered_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_ordered_value_traits_for_finding_kind() {
        implements_ordered_value_traits::<FindingKind>();
    }

    #[test]
    fn should_return_the_kind_when_its_name_is_given() {
        let name = "RollForwardMismatch";

        let expected_result = Some(FindingKind::RollForwardMismatch);

        let result = FindingKind::from_name(name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_no_kind_has_the_name() {
        let name = "rollUpMismatch";

        let expected_result = None;

        let result = FindingKind::from_name(name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_equal_rows_when_rows_pass_through_a_batch() {
        let rows = sample_rows();

        let expected_result = sample_rows();

        let batch = FindingRow::to_batch(&rows)
            .expect("Given rows whose values fit their columns, the batch should always build");
        let result = FindingRow::from_batch(&batch, FindingRow::SCHEMA_VERSION)
            .expect("Given a batch that `to_batch` built, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_match_the_checked_in_snapshot_for_the_schema_of_version_1() {
        let schema = FindingRow::schema(1).expect("Given version 1, the schema should exist");

        let expected_result = include_str!("schema_v1.txt");

        let result = schema_text(&schema);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_no_field_after_a_folder_label_for_findings() {
        let schema = FindingRow::schema(FindingRow::SCHEMA_VERSION)
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
        let batch = FindingRow::to_batch(&sample_rows())
            .expect("Given rows whose values fit their columns, the batch should always build");

        let expected_result = Err(ArrowRowError::UnknownSchemaVersion {
            dataset: "findings",
            version: 2,
        });

        let result = FindingRow::from_batch(&batch, 2);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_rows_when_the_batch_holds_no_rows() {
        let rows: Vec<FindingRow> = Vec::new();

        let expected_result: Vec<FindingRow> = Vec::new();

        let batch =
            FindingRow::to_batch(&rows).expect("Given no rows, the batch should always build");
        let result = FindingRow::from_batch(&batch, FindingRow::SCHEMA_VERSION)
            .expect("Given a batch that `to_batch` built, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_periods_are_not_sorted() {
        let mut row = sample_rows().remove(1);
        row.periods.reverse();
        let rows = vec![row];

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "findings",
            column: "periods",
            row: 0,
            reason: "not sorted by 'period_end', then 'quarters'".to_owned(),
        });

        let result = FindingRow::to_batch(&rows);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_kind_is_unknown() {
        let batch = batch_with(0, "kind", Arc::new(StringArray::from(vec!["Unmapped"])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "findings",
            column: "kind",
            row: 0,
            reason: "unknown kind 'Unmapped'".to_owned(),
        });

        let result = FindingRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_step_is_unknown() {
        let batch = batch_with(0, "step", Arc::new(StringArray::from(vec!["parse"])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "findings",
            column: "step",
            row: 0,
            reason: "unknown step 'parse'".to_owned(),
        });

        let result = FindingRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_filing_row_has_a_change() {
        let batch = batch_with(0, "change", Arc::new(StringArray::from(vec!["new"])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "findings",
            column: "change",
            row: 0,
            reason: "a value in a filing row".to_owned(),
        });

        let result = FindingRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_series_row_has_no_computed_at() {
        let batch = batch_with(
            1,
            "computed_at",
            Arc::new(TimestampMillisecondArray::from(vec![None]).with_timezone(UTC_TIME_ZONE)),
        );

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "findings",
            column: "computed_at",
            row: 0,
            reason: "null in a series row".to_owned(),
        });

        let result = FindingRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_filing_row_has_a_computed_at() {
        let batch = batch_with(
            0,
            "computed_at",
            Arc::new(
                TimestampMillisecondArray::from(vec![Some(1_793_966_700_000)])
                    .with_timezone(UTC_TIME_ZONE),
            ),
        );

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "findings",
            column: "computed_at",
            row: 0,
            reason: "a value in a filing row".to_owned(),
        });

        let result = FindingRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_a_series_row_has_no_change() {
        let batch = batch_with(1, "change", Arc::new(StringArray::from(vec![None::<&str>])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "findings",
            column: "change",
            row: 0,
            reason: "null in a series row".to_owned(),
        });

        let result = FindingRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_change_is_unknown() {
        let batch = batch_with(1, "change", Arc::new(StringArray::from(vec!["closed"])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "findings",
            column: "change",
            row: 0,
            reason: "unknown change 'closed'".to_owned(),
        });

        let result = FindingRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_confidence_is_unknown() {
        let batch = batch_with(
            2,
            "confidence",
            Arc::new(StringArray::from(vec!["synonym"])),
        );

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "findings",
            column: "confidence",
            row: 0,
            reason: "unknown confidence 'synonym'".to_owned(),
        });

        let result = FindingRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_filing_id_breaks_its_form() {
        let batch = batch_with(
            0,
            "filing_id",
            Arc::new(StringArray::from(vec!["0000320193"])),
        );

        let expected_result = Some("filing_id");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_confirming_filing_breaks_its_form() {
        let batch = batch_with(1, "confirmed_by", Arc::new(StringArray::from(vec!["x"])));

        let expected_result = Some("confirmed_by");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_entity_id_is_empty() {
        let batch = batch_with(0, "entity_id", Arc::new(StringArray::from(vec![""])));

        let expected_result = Some("entity_id");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_periods_when_a_period_end_lies_past_the_dates_of_jiff() {
        let period_ends = Date32Array::from(vec![20_358, i32::MAX]);

        let expected_result = Some("periods");

        let result = failing_column(&batch_with_period_ends(period_ends));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_computed_at_lies_past_the_times_of_jiff() {
        let batch = batch_with(
            1,
            "computed_at",
            Arc::new(TimestampMillisecondArray::from(vec![i64::MAX]).with_timezone(UTC_TIME_ZONE)),
        );

        let expected_result = Some("computed_at");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_transform_version_is_not_a_number() {
        let batch = batch_with(
            0,
            "transform_version",
            Arc::new(StringArray::from(vec!["v1"])),
        );

        let expected_result = Some("transform_version");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_same_digest_bytes_when_only_the_run_ids_and_computed_at_differ() {
        let first = sample_rows().remove(1);
        let mut second = first.clone();
        second.run.execution_id = Uuid::from_u128(2);
        second.run.loaded_at = Timestamp::UNIX_EPOCH;
        second.step = FindingStep::Series {
            change: FindingChange::New,
            computed_at: Timestamp::UNIX_EPOCH,
        };
        let mut first_bytes = ByteForm::new();
        first.write_row(&mut first_bytes);

        let expected_result = first_bytes;

        let mut result = ByteForm::new();
        second.write_row(&mut result);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_filing_id_as_the_digest_group_when_the_row_has_a_filing() {
        let row = sample_rows().remove(0);

        // marker 01 | length 00000018 | text sec:0000320193-26-000001
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x18sec:0000320193-26-000001";

        let mut byte_form = ByteForm::new();
        row.write_group_id(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_entity_as_the_digest_group_when_the_row_has_no_filing() {
        let row = sample_rows().remove(2);

        // entity_scheme | entity_id
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x16http://www.sec.gov/CIK\
\x01\x00\x00\x00\x0a0000320193";

        let mut byte_form = ByteForm::new();
        row.write_group_id(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_key_of_the_finding_when_the_row_key_is_written() {
        let row = sample_rows().remove(3);

        // kind | entity_scheme | entity_id | concepts empty | periods empty | filing_id null |
        // subject null
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x03Gap\
\x01\x00\x00\x00\x16http://www.sec.gov/CIK\
\x01\x00\x00\x00\x0a0000320193\
\x01\x00\x00\x00\x00\
\x01\x00\x00\x00\x00\
\x00\
\x00";

        let mut byte_form = ByteForm::new();
        row.write_row_key(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_every_column_but_the_run_ids_and_computed_at_in_schema_order_as_the_row() {
        let row = sample_rows().remove(2);

        // kind | entity_scheme | entity_id | concepts: list of 2 | periods: list of 1, struct of
        // 2 (period_end, quarters 0) | subject null | reason | filing_id null | confirmed_by null
        // | confidence | step | change | transform_version 1
        let expected_result: &[u8] = b"\x01\x00\x00\x00\x0eRollUpMismatch\
\x01\x00\x00\x00\x16http://www.sec.gov/CIK\
\x01\x00\x00\x00\x0a0000320193\
\x01\x00\x00\x00\x02\
\x01\x00\x00\x00\x0bLiabilities\
\x01\x00\x00\x00\x12LiabilitiesCurrent\
\x01\x00\x00\x00\x01\
\x01\x00\x00\x00\x02\
\x01\x00\x00\x00\x0a2026-09-26\
\x01\x00\x00\x00\x010\
\x00\
\x01\x00\x00\x00\x0bliabilities\
\x00\
\x00\
\x01\x00\x00\x00\x07Synonym\
\x01\x00\x00\x00\x06series\
\x01\x00\x00\x00\x07changed\
\x01\x00\x00\x00\x011";

        let mut byte_form = ByteForm::new();
        row.write_row(&mut byte_form);
        let result = byte_form.as_bytes();

        assert_eq!(result, expected_result);
    }
}
