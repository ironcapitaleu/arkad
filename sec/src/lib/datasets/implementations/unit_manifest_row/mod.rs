//! # Unit Manifest Row
//!
//! Provides [`UnitManifestRow`], one row of the dataset `unit_manifests`: the manifest of one unit,
//! with its inputs, its files, and the job that wrote it.

use std::str::FromStr;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, ListArray, RecordBatch, StringArray, StructArray, TimestampMillisecondArray,
    UInt16Array, UInt64Array,
};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::{DataType, Field, FieldRef, Fields, Schema, SchemaRef};
use jiff::Timestamp;
use storage::implementations::repository::filesystem::IngestionMode;
use storage::shared::Sha256Digest;
use uuid::Uuid;

use crate::datasets::columns::{
    BatchColumns, failed_batch, ingestion_mode_from_name, item_range, nullable, timestamp_array,
    timestamp_from_milliseconds, timestamp_type, unexpected_value,
};
use crate::datasets::{ArrowRow, ArrowRowError, SourceKind};

/// One row of `unit_manifests`: the manifest of one unit of a dataset.
///
/// A reader finds the current files of a dataset through these rows. The writer writes the row last.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnitManifestRow {
    /// The dataset of the unit, for example `as_is_facts`.
    pub dataset: String,
    /// The schema version of the unit's files.
    pub schema_version: u16,
    /// The source of the unit. `None` for the series datasets and `taxonomy_elements`.
    pub source: Option<SourceKind>,
    /// The kind of the unit.
    pub unit_kind: UnitKind,
    /// The key of the unit, for example `2024q1` for a Notes file.
    pub unit_key: String,
    /// The SHA-256 over the sorted list of input keys and digests.
    pub input_digest: Sha256Digest,
    /// Each raw document or dataset file that the unit read.
    pub inputs: Vec<UnitInput>,
    /// The version of the transform output.
    pub transform_version: u32,
    /// The Parquet files of the unit, in the order of their part number.
    pub files: Vec<UnitFile>,
    /// The rows of the unit, the sum of the rows of its files.
    pub row_count: u64,
    /// The rows of each input table that maps row to row.
    pub source_rows: Vec<TableRows>,
    /// How the job split a Notes file. `None` for the other units.
    pub split: Option<NotesSplit>,
    /// The rows the parser dropped on purpose, by reason.
    pub dropped_rows: Vec<DroppedRows>,
    /// The SHA-256 over the digest groups of the unit's rows.
    pub content_digest: Sha256Digest,
    /// The start time of the series run of a series unit. `None` for the other units. The part
    /// below one millisecond does not reach the batch.
    pub computed_at: Option<Timestamp>,
    /// The task count of the series run of a series unit. `None` for the other units.
    pub run_tasks: Option<u16>,
    /// The job that wrote the unit.
    pub execution_id: Uuid,
    /// The ingestion mode of the job.
    pub ingestion_mode: IngestionMode,
    /// The time of the write. The part below one millisecond does not reach the batch.
    pub written_at: Timestamp,
}

/// The kind of a unit: the text of the `unit_kind` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnitKind {
    /// All filings of one Notes file.
    NotesFile,
    /// All new filings of one filed date.
    FilingDay,
    /// The full series output of every entity.
    SeriesBase,
    /// The changed rows of one series run, for the buckets of one task.
    SeriesDelta,
    /// The elements of one taxonomy release.
    Release,
}

impl UnitKind {
    /// Returns the text of the `unit_kind` column, for example `notes_file`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotesFile => "notes_file",
            Self::FilingDay => "filing_day",
            Self::SeriesBase => "series_base",
            Self::SeriesDelta => "series_delta",
            Self::Release => "release",
        }
    }

    /// Returns the kind whose text is `text`, or `None` if no kind has it.
    #[must_use]
    pub fn from_name(text: &str) -> Option<Self> {
        [
            Self::NotesFile,
            Self::FilingDay,
            Self::SeriesBase,
            Self::SeriesDelta,
            Self::Release,
        ]
        .into_iter()
        .find(|kind| kind.as_str() == text)
    }
}

/// How the job split a Notes file: the text of the `split` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NotesSplit {
    /// One pass over the file, split by filing.
    Stream,
    /// Several passes over the file, one per set of entity buckets.
    Buckets,
}

impl NotesSplit {
    /// Returns the text of the `split` column: `stream` or `buckets`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stream => "stream",
            Self::Buckets => "buckets",
        }
    }

    /// Returns the split whose text is `text`, or `None` if no split has it.
    #[must_use]
    pub fn from_name(text: &str) -> Option<Self> {
        [Self::Stream, Self::Buckets]
            .into_iter()
            .find(|split| split.as_str() == text)
    }
}

/// One raw document or dataset file that a unit read: an item of the `inputs` column.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitInput {
    /// The key of the document in its store.
    pub key: String,
    /// The SHA-256 of the document.
    pub sha256: Sha256Digest,
    /// The URL of the request that fetched a raw document. `None` for a dataset file and the
    /// vendored release.
    pub requested_url: Option<String>,
    /// The time the document store wrote a raw document. `None` for a dataset file and the vendored
    /// release. The part below one millisecond does not reach the batch.
    pub produced_at: Option<Timestamp>,
}

/// One Parquet file of a unit: an item of the `files` column.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitFile {
    /// The path of the file below the bucket, for example
    /// `as_is_facts/source=notes/unit_key=2024q1/<digest>-v1/<execution_id>-0/part-0.parquet`.
    pub path: String,
    /// The size of the file.
    pub bytes: u64,
    /// The CRC32C of the file, as GCS stores it.
    pub crc32c: String,
    /// The rows of the file.
    pub row_count: u64,
}

/// The rows of one input table of a unit: an item of the `source_rows` column.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TableRows {
    /// The input table, for example `num.tsv`.
    pub table: String,
    /// The rows of the table.
    pub rows: u64,
}

/// The rows the parser dropped for one reason: an item of the `dropped_rows` column.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DroppedRows {
    /// The reason, for example `UnmappedRow`.
    pub reason: String,
    /// The rows dropped for the reason.
    pub rows: u64,
}

impl ArrowRow for UnitManifestRow {
    const DATASET: &'static str = "unit_manifests";
    const SCHEMA_VERSION: u16 = 1;

    fn schema(version: u16) -> Option<SchemaRef> {
        (version == 1).then(schema_v1)
    }

    fn to_batch(rows: &[Self]) -> Result<RecordBatch, ArrowRowError> {
        for (index, row) in rows.iter().enumerate() {
            check_row_count(row.row_count, &row.files)
                .map_err(|reason| unexpected_value(Self::DATASET, "row_count", index, reason))?;
        }
        let text = |value: fn(&Self) -> Option<String>| -> ArrayRef {
            Arc::new(StringArray::from_iter(rows.iter().map(value)))
        };
        let columns: Vec<ArrayRef> = vec![
            text(|row| Some(row.dataset.clone())),
            Arc::new(UInt16Array::from_iter_values(
                rows.iter().map(|row| row.schema_version),
            )),
            text(|row| row.source.map(|source| source.as_str().to_owned())),
            text(|row| Some(row.unit_kind.as_str().to_owned())),
            text(|row| Some(row.unit_key.clone())),
            text(|row| Some(row.input_digest.to_string())),
            inputs_array(rows)?,
            text(|row| Some(row.transform_version.to_string())),
            files_array(rows)?,
            Arc::new(UInt64Array::from_iter_values(
                rows.iter().map(|row| row.row_count),
            )),
            counts_array(
                "source_rows",
                rows.iter().map(|row| {
                    row.source_rows
                        .iter()
                        .map(|count| (&count.table, count.rows))
                }),
            )?,
            text(|row| row.split.map(|split| split.as_str().to_owned())),
            counts_array(
                "dropped_rows",
                rows.iter().map(|row| {
                    row.dropped_rows
                        .iter()
                        .map(|count| (&count.reason, count.rows))
                }),
            )?,
            text(|row| Some(row.content_digest.to_string())),
            timestamp_array(rows.iter().map(|row| row.computed_at)),
            Arc::new(UInt16Array::from_iter(rows.iter().map(|row| row.run_tasks))),
            text(|row| Some(row.execution_id.to_string())),
            text(|row| Some(row.ingestion_mode.to_string())),
            timestamp_array(rows.iter().map(|row| Some(row.written_at))),
        ];
        RecordBatch::try_new(schema_v1(), columns)
            .map_err(|error| failed_batch(Self::DATASET, &error))
    }

    fn from_batch(batch: &RecordBatch, version: u16) -> Result<Vec<Self>, ArrowRowError> {
        let columns = BatchColumns::check::<Self>(batch, version)?;
        let reader = UnitColumns::new(&columns)?;
        (0..columns.row_count())
            .map(|row| reader.row(row))
            .collect()
    }
}

/// Checks that `row_count` is the sum of the rows of the files.
///
/// Fails if the sum differs, or if it does not fit in a `u64`.
fn check_row_count(row_count: u64, files: &[UnitFile]) -> Result<(), String> {
    let file_rows = files
        .iter()
        .try_fold(0_u64, |sum, file| sum.checked_add(file.row_count))
        .ok_or_else(|| "the rows of the files overflow 'u64'".to_owned())?;
    if row_count == file_rows {
        Ok(())
    } else {
        Err(format!("the files hold '{file_rows}' rows"))
    }
}

/// Returns the schema of version 1.
fn schema_v1() -> SchemaRef {
    let text = |name: &str, nullable: bool| Field::new(name, DataType::Utf8, nullable);
    let count = |name: &str| Field::new(name, DataType::UInt64, false);
    let fields = vec![
        text("dataset", false),
        Field::new("schema_version", DataType::UInt16, false),
        text("source", true),
        text("unit_kind", false),
        text("unit_key", false),
        text("input_digest", false),
        Field::new(
            "inputs",
            DataType::List(struct_item(vec![
                text("key", false),
                text("sha256", false),
                text("requested_url", true),
                Field::new("produced_at", timestamp_type(), true),
            ])),
            false,
        ),
        text("transform_version", false),
        Field::new(
            "files",
            DataType::List(struct_item(vec![
                text("path", false),
                count("bytes"),
                text("crc32c", false),
                count("row_count"),
            ])),
            false,
        ),
        count("row_count"),
        Field::new(
            "source_rows",
            DataType::List(struct_item(vec![text("table", false), count("rows")])),
            false,
        ),
        text("split", true),
        Field::new(
            "dropped_rows",
            DataType::List(struct_item(vec![text("reason", false), count("rows")])),
            false,
        ),
        text("content_digest", false),
        Field::new("computed_at", timestamp_type(), true),
        Field::new("run_tasks", DataType::UInt16, true),
        text("execution_id", false),
        text("ingestion_mode", false),
        Field::new("written_at", timestamp_type(), false),
    ];
    Arc::new(Schema::new(fields))
}

/// Returns the item field of a list column of structs with the fields.
fn struct_item(fields: Vec<Field>) -> FieldRef {
    Arc::new(Field::new_list_field(
        DataType::Struct(Fields::from(fields)),
        false,
    ))
}

/// Returns the item field of a list column of the schema.
fn item_of(column: &str) -> FieldRef {
    match schema_v1().field_with_name(column).map(Field::data_type) {
        Ok(DataType::List(item)) => Arc::clone(item),
        _ => unreachable!("Given a list column of the schema of `unit_manifests`"),
    }
}

/// Returns a list array of structs. Row `n` holds the next `lengths[n]` items of the columns.
fn struct_list(
    column: &str,
    lengths: Vec<usize>,
    item_columns: Vec<ArrayRef>,
) -> Result<ArrayRef, ArrowRowError> {
    let item = item_of(column);
    let DataType::Struct(fields) = item.data_type() else {
        unreachable!("Given the item field of a list column of structs")
    };
    let items = StructArray::try_new(Fields::clone(fields), item_columns, None)
        .map_err(|error| failed_batch(UnitManifestRow::DATASET, &error))?;
    let list = ListArray::try_new(
        item,
        OffsetBuffer::from_lengths(lengths),
        Arc::new(items),
        None,
    )
    .map_err(|error| failed_batch(UnitManifestRow::DATASET, &error))?;
    Ok(Arc::new(list))
}

/// Returns the `inputs` array: per row, the documents that the unit read.
fn inputs_array(rows: &[UnitManifestRow]) -> Result<ArrayRef, ArrowRowError> {
    let inputs = || rows.iter().flat_map(|row| &row.inputs);
    struct_list(
        "inputs",
        rows.iter().map(|row| row.inputs.len()).collect(),
        vec![
            Arc::new(StringArray::from_iter_values(
                inputs().map(|input| &input.key),
            )),
            Arc::new(StringArray::from_iter_values(
                inputs().map(|input| input.sha256.to_string()),
            )),
            Arc::new(StringArray::from_iter(
                inputs().map(|input| input.requested_url.as_deref()),
            )),
            timestamp_array(inputs().map(|input| input.produced_at)),
        ],
    )
}

/// Returns the `files` array: per row, the Parquet files of the unit.
fn files_array(rows: &[UnitManifestRow]) -> Result<ArrayRef, ArrowRowError> {
    let files = || rows.iter().flat_map(|row| &row.files);
    struct_list(
        "files",
        rows.iter().map(|row| row.files.len()).collect(),
        vec![
            Arc::new(StringArray::from_iter_values(
                files().map(|file| &file.path),
            )),
            Arc::new(UInt64Array::from_iter_values(
                files().map(|file| file.bytes),
            )),
            Arc::new(StringArray::from_iter_values(
                files().map(|file| &file.crc32c),
            )),
            Arc::new(UInt64Array::from_iter_values(
                files().map(|file| file.row_count),
            )),
        ],
    )
}

/// Returns a `source_rows` or `dropped_rows` array from the name and the rows of each count.
fn counts_array<'r, R, C>(column: &str, rows: R) -> Result<ArrayRef, ArrowRowError>
where
    R: Iterator<Item = C> + Clone,
    C: Iterator<Item = (&'r String, u64)>,
{
    let lengths = rows.clone().map(Iterator::count).collect();
    let (names, counts): (Vec<&String>, Vec<u64>) = rows.flatten().unzip();
    struct_list(
        column,
        lengths,
        vec![
            Arc::new(StringArray::from_iter_values(names)),
            Arc::new(UInt64Array::from(counts)),
        ],
    )
}

/// The arrays of the columns of one batch of `unit_manifests`.
struct UnitColumns<'a> {
    dataset: &'static str,
    datasets: &'a StringArray,
    schema_versions: &'a UInt16Array,
    sources: &'a StringArray,
    unit_kinds: &'a StringArray,
    unit_keys: &'a StringArray,
    input_digests: &'a StringArray,
    inputs: &'a ListArray,
    input_keys: &'a StringArray,
    input_sha256s: &'a StringArray,
    input_urls: &'a StringArray,
    input_produced_at: &'a TimestampMillisecondArray,
    transform_versions: &'a StringArray,
    files: &'a ListArray,
    file_paths: &'a StringArray,
    file_bytes: &'a UInt64Array,
    file_crc32cs: &'a StringArray,
    file_row_counts: &'a UInt64Array,
    row_counts: &'a UInt64Array,
    source_rows: CountColumns<'a>,
    splits: &'a StringArray,
    dropped_rows: CountColumns<'a>,
    content_digests: &'a StringArray,
    computed_at: &'a TimestampMillisecondArray,
    run_tasks: &'a UInt16Array,
    execution_ids: &'a StringArray,
    ingestion_modes: &'a StringArray,
    written_at: &'a TimestampMillisecondArray,
}

/// The arrays of a `source_rows` or `dropped_rows` column of one batch.
struct CountColumns<'a> {
    list: &'a ListArray,
    names: &'a StringArray,
    rows: &'a UInt64Array,
}

impl<'a> CountColumns<'a> {
    /// Takes the arrays of a count column whose name field is `name`.
    fn new(columns: &BatchColumns<'a>, column: &str, name: &str) -> Result<Self, ArrowRowError> {
        Ok(Self {
            list: columns.get(column)?,
            names: columns.item_field(column, name)?,
            rows: columns.item_field(column, "rows")?,
        })
    }

    /// Returns the name and the rows of each count of one row.
    fn counts(&self, row: usize) -> impl Iterator<Item = (String, u64)> + '_ {
        item_range(self.list, row)
            .map(|item| (self.names.value(item).to_owned(), self.rows.value(item)))
    }
}

impl<'a> UnitColumns<'a> {
    /// Takes the arrays of the columns from a batch that passed the schema check.
    fn new(columns: &BatchColumns<'a>) -> Result<Self, ArrowRowError> {
        Ok(Self {
            dataset: columns.dataset(),
            datasets: columns.get("dataset")?,
            schema_versions: columns.get("schema_version")?,
            sources: columns.get("source")?,
            unit_kinds: columns.get("unit_kind")?,
            unit_keys: columns.get("unit_key")?,
            input_digests: columns.get("input_digest")?,
            inputs: columns.get("inputs")?,
            input_keys: columns.item_field("inputs", "key")?,
            input_sha256s: columns.item_field("inputs", "sha256")?,
            input_urls: columns.item_field("inputs", "requested_url")?,
            input_produced_at: columns.item_field("inputs", "produced_at")?,
            transform_versions: columns.get("transform_version")?,
            files: columns.get("files")?,
            file_paths: columns.item_field("files", "path")?,
            file_bytes: columns.item_field("files", "bytes")?,
            file_crc32cs: columns.item_field("files", "crc32c")?,
            file_row_counts: columns.item_field("files", "row_count")?,
            row_counts: columns.get("row_count")?,
            source_rows: CountColumns::new(columns, "source_rows", "table")?,
            splits: columns.get("split")?,
            dropped_rows: CountColumns::new(columns, "dropped_rows", "reason")?,
            content_digests: columns.get("content_digest")?,
            computed_at: columns.get("computed_at")?,
            run_tasks: columns.get("run_tasks")?,
            execution_ids: columns.get("execution_id")?,
            ingestion_modes: columns.get("ingestion_mode")?,
            written_at: columns.get("written_at")?,
        })
    }

    /// Reads the unit of one row.
    fn row(&self, row: usize) -> Result<UnitManifestRow, ArrowRowError> {
        let fail = |column: &'static str, reason: String| {
            unexpected_value(self.dataset, column, row, reason)
        };
        let digest = |column: &'static str, text: &str| {
            Sha256Digest::from_str(text).map_err(|error| fail(column, error.to_string()))
        };
        let timestamp = |column: &'static str, milliseconds| {
            timestamp_from_milliseconds(milliseconds).map_err(|reason| fail(column, reason))
        };
        let files = self.files(row);
        let row_count = self.row_counts.value(row);
        check_row_count(row_count, &files).map_err(|reason| fail("row_count", reason))?;
        let unit_kind = self.unit_kinds.value(row);
        let mode = self.ingestion_modes.value(row);
        Ok(UnitManifestRow {
            dataset: self.datasets.value(row).to_owned(),
            schema_version: self.schema_versions.value(row),
            source: nullable(self.sources, row)
                .map(|text| {
                    SourceKind::from_name(text)
                        .ok_or_else(|| fail("source", "not 'notes' or 'instance'".to_owned()))
                })
                .transpose()?,
            unit_kind: UnitKind::from_name(unit_kind)
                .ok_or_else(|| fail("unit_kind", format!("unknown unit kind '{unit_kind}'")))?,
            unit_key: self.unit_keys.value(row).to_owned(),
            input_digest: digest("input_digest", self.input_digests.value(row))?,
            inputs: item_range(self.inputs, row)
                .map(|item| {
                    Ok(UnitInput {
                        key: self.input_keys.value(item).to_owned(),
                        sha256: digest("inputs", self.input_sha256s.value(item))?,
                        requested_url: nullable(self.input_urls, item).map(str::to_owned),
                        produced_at: nullable(self.input_produced_at, item)
                            .map(|milliseconds| timestamp("inputs", milliseconds))
                            .transpose()?,
                    })
                })
                .collect::<Result<_, ArrowRowError>>()?,
            transform_version: self
                .transform_versions
                .value(row)
                .parse()
                .map_err(|_| fail("transform_version", "not a base-10 number".to_owned()))?,
            files,
            row_count,
            source_rows: self
                .source_rows
                .counts(row)
                .map(|(table, rows)| TableRows { table, rows })
                .collect(),
            split: nullable(self.splits, row)
                .map(|text| {
                    NotesSplit::from_name(text)
                        .ok_or_else(|| fail("split", "not 'stream' or 'buckets'".to_owned()))
                })
                .transpose()?,
            dropped_rows: self
                .dropped_rows
                .counts(row)
                .map(|(reason, rows)| DroppedRows { reason, rows })
                .collect(),
            content_digest: digest("content_digest", self.content_digests.value(row))?,
            computed_at: nullable(self.computed_at, row)
                .map(|milliseconds| timestamp("computed_at", milliseconds))
                .transpose()?,
            run_tasks: nullable(self.run_tasks, row),
            execution_id: Uuid::parse_str(self.execution_ids.value(row))
                .map_err(|error| fail("execution_id", error.to_string()))?,
            ingestion_mode: ingestion_mode_from_name(mode).ok_or_else(|| {
                fail(
                    "ingestion_mode",
                    "not 'backfill' or 'incremental'".to_owned(),
                )
            })?,
            written_at: timestamp("written_at", self.written_at.value(row))?,
        })
    }

    /// Reads the files of one row.
    fn files(&self, row: usize) -> Vec<UnitFile> {
        item_range(self.files, row)
            .map(|item| UnitFile {
                path: self.file_paths.value(item).to_owned(),
                bytes: self.file_bytes.value(item),
                crc32c: self.file_crc32cs.value(item).to_owned(),
                row_count: self.file_row_counts.value(item),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use arrow::array::Array;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::datasets::{UNIT_MANIFEST_FOLDER_LABELS, UTC_TIME_ZONE, schema_text};

    /// A time of a write.
    fn written_at() -> Timestamp {
        Timestamp::from_millisecond(1_793_966_700_123).expect(
            "Given a hardcoded time in the range of jiff, the creation should always succeed",
        )
    }

    /// A unit of `taxonomy_elements` with every list empty and every nullable column empty.
    fn release_unit() -> UnitManifestRow {
        UnitManifestRow {
            dataset: "taxonomy_elements".to_owned(),
            schema_version: 1,
            source: None,
            unit_kind: UnitKind::Release,
            unit_key: "us-gaap/2026".to_owned(),
            input_digest: Sha256Digest::of(b"release inputs"),
            inputs: Vec::new(),
            transform_version: 1,
            files: Vec::new(),
            row_count: 0,
            source_rows: Vec::new(),
            split: None,
            dropped_rows: Vec::new(),
            content_digest: Sha256Digest::of(b"release rows"),
            computed_at: None,
            run_tasks: None,
            execution_id: Uuid::from_u128(1),
            ingestion_mode: IngestionMode::Backfill,
            written_at: written_at(),
        }
    }

    /// The fixture rows: a Notes unit with every column set, a series unit that read a dataset
    /// file with a non-ASCII key, a release unit with every list and nullable column empty, and
    /// a filing day unit of the instance source.
    fn sample_rows() -> Vec<UnitManifestRow> {
        let notes = UnitManifestRow {
            dataset: "as_is_facts".to_owned(),
            source: Some(SourceKind::Notes),
            unit_kind: UnitKind::NotesFile,
            unit_key: "2024q1".to_owned(),
            inputs: vec![UnitInput {
                key: "sec/notes/2024q1_notes.zip".to_owned(),
                sha256: Sha256Digest::of(b"notes zip"),
                requested_url: Some(
                    "https://www.sec.gov/files/dera/data/financial-statement-and-notes-data-sets/2024q1_notes.zip"
                        .to_owned(),
                ),
                produced_at: Some(written_at()),
            }],
            files: vec![
                UnitFile {
                    path: "as_is_facts/source=notes/unit_key=2024q1/0123456789abcdef-v1/00000000-0000-0000-0000-000000000001-0/part-0.parquet"
                        .to_owned(),
                    bytes: 52_428_800,
                    crc32c: "AAAAAA==".to_owned(),
                    row_count: 3,
                },
                UnitFile {
                    path: "as_is_facts/source=notes/unit_key=2024q1/0123456789abcdef-v1/00000000-0000-0000-0000-000000000001-0/part-1.parquet"
                        .to_owned(),
                    bytes: 1_024,
                    crc32c: "yZRlqg==".to_owned(),
                    row_count: 4,
                },
            ],
            row_count: 7,
            source_rows: vec![
                TableRows {
                    table: "num.tsv".to_owned(),
                    rows: 5,
                },
                TableRows {
                    table: "txt.tsv".to_owned(),
                    rows: 3,
                },
            ],
            split: Some(NotesSplit::Stream),
            dropped_rows: vec![DroppedRows {
                reason: "UnmappedRow".to_owned(),
                rows: 1,
            }],
            ..release_unit()
        };
        let series = UnitManifestRow {
            dataset: "findings".to_owned(),
            unit_kind: UnitKind::SeriesDelta,
            unit_key: "2026-11-06T0600Z-t0".to_owned(),
            inputs: vec![UnitInput {
                key: "candidates/source=instance/unit_key=Société-株式会社/part-0.parquet"
                    .to_owned(),
                sha256: Sha256Digest::of(b"candidates part"),
                requested_url: None,
                produced_at: None,
            }],
            files: vec![UnitFile {
                path: "findings/unit_key=2026-11-06T0600Z-t0/part-0.parquet".to_owned(),
                bytes: u64::MAX,
                crc32c: "Q9h3tA==".to_owned(),
                row_count: 2,
            }],
            row_count: 2,
            computed_at: Some(written_at()),
            run_tasks: Some(64),
            ingestion_mode: IngestionMode::Incremental,
            ..release_unit()
        };
        let filing_day = UnitManifestRow {
            dataset: "filings".to_owned(),
            source: Some(SourceKind::Instance),
            unit_kind: UnitKind::FilingDay,
            unit_key: "2026-11-06".to_owned(),
            ingestion_mode: IngestionMode::Incremental,
            ..release_unit()
        };
        vec![notes, series, release_unit(), filing_day]
    }

    /// Returns the batch of fixture row `row`, with the column `name` replaced by `column`.
    fn batch_with(row: usize, name: &str, column: ArrayRef) -> RecordBatch {
        let batch = UnitManifestRow::to_batch(&sample_rows()[row..=row])
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

    /// Returns the batch of the Notes fixture row, with the field `field` of the items of the list
    /// column `column` replaced by `items`.
    fn batch_with_item_field(column: &str, field: &str, items: ArrayRef) -> RecordBatch {
        let batch = UnitManifestRow::to_batch(&sample_rows()[..1])
            .expect("Given rows whose values fit their columns, the batch should always build");
        let list = batch
            .column_by_name(column)
            .and_then(|array| array.as_any().downcast_ref::<ListArray>())
            .expect("Given a list column of the schema, the lookup should always succeed");
        let structs = list
            .values()
            .as_any()
            .downcast_ref::<StructArray>()
            .expect("Given a list column of structs, its items should always be structs");
        let (index, _) = structs
            .fields()
            .find(field)
            .expect("Given a field of the struct, the lookup should always succeed");
        let mut item_columns = structs.columns().to_vec();
        item_columns[index] = items;
        let structs = StructArray::try_new(structs.fields().clone(), item_columns, None)
            .expect("Given a field of the same type and length, the struct should always build");
        let list = ListArray::try_new(
            item_of(column),
            list.offsets().clone(),
            Arc::new(structs),
            None,
        )
        .expect("Given the offsets of the same list, the list should always build");
        batch_with(0, column, Arc::new(list))
    }

    /// Returns the column that the error of reading the batch names.
    fn failing_column(batch: &RecordBatch) -> Option<&'static str> {
        match UnitManifestRow::from_batch(batch, 1) {
            Err(ArrowRowError::UnexpectedColumnValue { column, .. }) => Some(column),
            _ => None,
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_unit_manifest_row() {
        implements_auto_traits::<UnitManifestRow>();
    }

    const fn implements_value_traits<T: Debug + Clone + PartialEq + Eq + Hash>() {}
    #[test]
    const fn should_implement_value_traits_for_unit_manifest_row() {
        implements_value_traits::<UnitManifestRow>();
    }

    #[test]
    fn should_write_the_texts_of_the_contract_for_every_unit_kind() {
        let kinds = [
            UnitKind::NotesFile,
            UnitKind::FilingDay,
            UnitKind::SeriesBase,
            UnitKind::SeriesDelta,
            UnitKind::Release,
        ];

        let expected_result = [
            "notes_file",
            "filing_day",
            "series_base",
            "series_delta",
            "release",
        ];

        let result = kinds.map(UnitKind::as_str);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_every_unit_kind_back_from_its_text() {
        let texts = [
            "notes_file",
            "filing_day",
            "series_base",
            "series_delta",
            "release",
        ];

        let expected_result = [
            Some(UnitKind::NotesFile),
            Some(UnitKind::FilingDay),
            Some(UnitKind::SeriesBase),
            Some(UnitKind::SeriesDelta),
            Some(UnitKind::Release),
        ];

        let result = texts.map(UnitKind::from_name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_texts_of_the_contract_for_every_split() {
        let splits = [NotesSplit::Stream, NotesSplit::Buckets];

        let expected_result = ["stream", "buckets"];

        let result = splits.map(NotesSplit::as_str);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_read_every_split_back_from_its_text() {
        let texts = ["stream", "buckets"];

        let expected_result = [Some(NotesSplit::Stream), Some(NotesSplit::Buckets)];

        let result = texts.map(NotesSplit::from_name);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_row_count_differs_from_the_rows_of_the_files_on_write() {
        let mut rows = sample_rows();
        rows[1].row_count = 3;

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "unit_manifests",
            column: "row_count",
            row: 1,
            reason: "the files hold '2' rows".to_owned(),
        });

        let result = UnitManifestRow::to_batch(&rows);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_row_when_a_later_row_of_the_batch_breaks_a_rule() {
        let batch = UnitManifestRow::to_batch(&sample_rows()[..2])
            .expect("Given rows whose values fit their columns, the batch should always build");
        let index = batch
            .schema()
            .index_of("row_count")
            .expect("Given a column of the schema of version 1, the lookup should always succeed");
        let mut columns = batch.columns().to_vec();
        columns[index] = Arc::new(UInt64Array::from(vec![7, 3]));
        let batch = RecordBatch::try_new(batch.schema(), columns)
            .expect("Given a column of the same type, the batch should always build");

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "unit_manifests",
            column: "row_count",
            row: 1,
            reason: "the files hold '2' rows".to_owned(),
        });

        let result = UnitManifestRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_rows_of_the_files_overflow_u64() {
        let batch = batch_with_item_field(
            "files",
            "row_count",
            Arc::new(UInt64Array::from(vec![u64::MAX, 1])),
        );

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "unit_manifests",
            column: "row_count",
            row: 0,
            reason: "the rows of the files overflow 'u64'".to_owned(),
        });

        let result = UnitManifestRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_equal_rows_when_rows_pass_through_a_batch() {
        let rows = sample_rows();

        let expected_result = sample_rows();

        let batch = UnitManifestRow::to_batch(&rows)
            .expect("Given rows whose values fit their columns, the batch should always build");
        let result = UnitManifestRow::from_batch(&batch, UnitManifestRow::SCHEMA_VERSION)
            .expect("Given a batch that `to_batch` built, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_match_the_checked_in_snapshot_for_the_schema_of_version_1() {
        let schema = UnitManifestRow::schema(1).expect("Given version 1, the schema should exist");

        let expected_result = include_str!("schema_v1.txt");

        let result = schema_text(&schema);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_no_field_after_a_folder_label_for_unit_manifests() {
        let schema = UnitManifestRow::schema(UnitManifestRow::SCHEMA_VERSION)
            .expect("Given the version that `to_batch` writes, the schema should exist");

        let expected_result: Vec<&str> = Vec::new();

        let result: Vec<&str> = schema
            .fields()
            .iter()
            .map(|field| field.name().as_str())
            .filter(|name| UNIT_MANIFEST_FOLDER_LABELS.contains(name))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_schema_version_is_unknown() {
        let batch = UnitManifestRow::to_batch(&sample_rows())
            .expect("Given rows whose values fit their columns, the batch should always build");

        let expected_result = Err(ArrowRowError::UnknownSchemaVersion {
            dataset: "unit_manifests",
            version: 2,
        });

        let result = UnitManifestRow::from_batch(&batch, 2);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_rows_when_the_batch_holds_no_rows() {
        let rows: Vec<UnitManifestRow> = Vec::new();

        let expected_result: Vec<UnitManifestRow> = Vec::new();

        let batch =
            UnitManifestRow::to_batch(&rows).expect("Given no rows, the batch should always build");
        let result = UnitManifestRow::from_batch(&batch, UnitManifestRow::SCHEMA_VERSION)
            .expect("Given a batch that `to_batch` built, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_row_count_differs_from_the_rows_of_the_files() {
        let batch = batch_with(0, "row_count", Arc::new(UInt64Array::from(vec![8])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "unit_manifests",
            column: "row_count",
            row: 0,
            reason: "the files hold '7' rows".to_owned(),
        });

        let result = UnitManifestRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_unit_kind_is_unknown() {
        let batch = batch_with(0, "unit_kind", Arc::new(StringArray::from(vec!["notes"])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "unit_manifests",
            column: "unit_kind",
            row: 0,
            reason: "unknown unit kind 'notes'".to_owned(),
        });

        let result = UnitManifestRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_source_is_unknown() {
        let batch = batch_with(0, "source", Arc::new(StringArray::from(vec!["zip"])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "unit_manifests",
            column: "source",
            row: 0,
            reason: "not 'notes' or 'instance'".to_owned(),
        });

        let result = UnitManifestRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_split_is_unknown() {
        let batch = batch_with(0, "split", Arc::new(StringArray::from(vec!["passes"])));

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "unit_manifests",
            column: "split",
            row: 0,
            reason: "not 'stream' or 'buckets'".to_owned(),
        });

        let result = UnitManifestRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_ingestion_mode_is_unknown() {
        let batch = batch_with(
            0,
            "ingestion_mode",
            Arc::new(StringArray::from(vec!["daily"])),
        );

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "unit_manifests",
            column: "ingestion_mode",
            row: 0,
            reason: "not 'backfill' or 'incremental'".to_owned(),
        });

        let result = UnitManifestRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_when_the_transform_version_is_not_a_number() {
        let batch = batch_with(
            0,
            "transform_version",
            Arc::new(StringArray::from(vec!["v1"])),
        );

        let expected_result = Err(ArrowRowError::UnexpectedColumnValue {
            dataset: "unit_manifests",
            column: "transform_version",
            row: 0,
            reason: "not a base-10 number".to_owned(),
        });

        let result = UnitManifestRow::from_batch(&batch, 1);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_input_digest_is_not_a_digest() {
        let batch = batch_with(0, "input_digest", Arc::new(StringArray::from(vec!["abc"])));

        let expected_result = Some("input_digest");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_content_digest_is_not_a_digest() {
        let batch = batch_with(
            0,
            "content_digest",
            Arc::new(StringArray::from(vec!["abc"])),
        );

        let expected_result = Some("content_digest");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_inputs_when_the_digest_of_an_input_is_not_a_digest() {
        let batch =
            batch_with_item_field("inputs", "sha256", Arc::new(StringArray::from(vec!["abc"])));

        let expected_result = Some("inputs");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_inputs_when_the_produced_at_of_an_input_lies_past_the_times_of_jiff() {
        let batch = batch_with_item_field(
            "inputs",
            "produced_at",
            Arc::new(TimestampMillisecondArray::from(vec![i64::MAX]).with_timezone(UTC_TIME_ZONE)),
        );

        let expected_result = Some("inputs");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_column_when_the_execution_id_is_not_a_uuid() {
        let batch = batch_with(
            0,
            "execution_id",
            Arc::new(StringArray::from(vec!["run-1"])),
        );

        let expected_result = Some("execution_id");

        let result = failing_column(&batch);

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
    fn should_name_the_column_when_written_at_lies_past_the_times_of_jiff() {
        let batch = batch_with(
            0,
            "written_at",
            Arc::new(TimestampMillisecondArray::from(vec![i64::MAX]).with_timezone(UTC_TIME_ZONE)),
        );

        let expected_result = Some("written_at");

        let result = failing_column(&batch);

        assert_eq!(result, expected_result);
    }
}
