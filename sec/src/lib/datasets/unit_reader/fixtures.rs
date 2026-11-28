//! # Fixtures
//!
//! Provides the store, the times, the `unit_manifests` rows, and the row types of the tests of the
//! reader.

use std::sync::Arc;

use arrow::array::{Array, ArrayRef, RecordBatch, StringArray, UInt8Array};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use bytes::Bytes;
use jiff::Timestamp;
use parquet::arrow::ArrowWriter;
use storage::implementations::repository::filesystem::IngestionMode;
use storage::implementations::repository::write_once::WriteOnceFilesystemRepository;
use storage::shared::Sha256Digest;
use tempfile::TempDir;
use uuid::Uuid;

use crate::datasets::implementations::UnitKind;
use crate::datasets::{
    ArrowRow, ArrowRowError, PartLocation, PartOrder, SourceKind, TaskAttempt, TaskLocation,
    UnitManifestRow, WrittenUnit, write_part, write_task_manifest,
};
use crate::shared::row_digest::ByteForm;

/// The dataset of the rows of the fixtures.
pub const DATASET: &str = "fake_rows";

/// Returns an empty store in a new temporary folder. The folder lives as long as the `TempDir`.
pub fn store() -> (TempDir, WriteOnceFilesystemRepository) {
    let directory = tempfile::tempdir()
        .expect("Given a writable temporary folder, the creation should always succeed");
    let store = WriteOnceFilesystemRepository::new(directory.path());
    (directory, store)
}

pub fn time(text: &str) -> Timestamp {
    text.parse()
        .expect("Given a valid RFC 3339 text, the time should always parse")
}

/// Returns the row of a `notes_file` unit of [`DATASET`] with no files.
pub fn notes_unit(unit_key: &str, execution: u128) -> UnitManifestRow {
    UnitManifestRow {
        dataset: DATASET.to_owned(),
        schema_version: 1,
        source: Some(SourceKind::Notes),
        unit_kind: UnitKind::NotesFile,
        unit_key: unit_key.to_owned(),
        input_digest: Sha256Digest::of(unit_key),
        inputs: Vec::new(),
        transform_version: 1,
        files: Vec::new(),
        row_count: 0,
        source_rows: Vec::new(),
        split: None,
        dropped_rows: Vec::new(),
        content_digest: Sha256Digest::of(""),
        computed_at: None,
        run_tasks: None,
        execution_id: Uuid::from_u128(execution),
        ingestion_mode: IngestionMode::Backfill,
        // The list file of the task sets the time of the write.
        written_at: Timestamp::UNIX_EPOCH,
    }
}

/// Returns the row of a series unit of [`DATASET`] with no files.
pub fn series_unit(
    unit_kind: UnitKind,
    unit_key: &str,
    execution: u128,
    computed_at: &str,
    run_tasks: u16,
) -> UnitManifestRow {
    UnitManifestRow {
        source: None,
        unit_kind,
        computed_at: Some(time(computed_at)),
        run_tasks: Some(run_tasks),
        ..notes_unit(unit_key, execution)
    }
}

/// Writes the list file of one task with the rows, at the time `now`.
///
/// The task index comes from the order of the calls in a test, and the attempt from `attempt`.
pub async fn write_units(
    store: &WriteOnceFilesystemRepository,
    task_index: u32,
    attempt: u32,
    rows: Vec<UnitManifestRow>,
    now: &str,
) {
    let attempt = TaskAttempt {
        number: attempt,
        is_bucket_split: false,
    };
    let task = TaskLocation {
        execution_id: rows
            .first()
            .expect("Given a test that writes a list file, the task should always hold a unit")
            .execution_id,
        task_index,
        attempt,
    };
    let units = rows
        .into_iter()
        .map(|row| WrittenUnit {
            row,
            current_written_at: None,
            attempt,
        })
        .collect();
    write_task_manifest(store, &task, units, time(now))
        .await
        .expect("Given a new task location, the list file should always be new");
}

/// Version 1 of a row of [`DATASET`]: a fact of a filing, without a unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeRowV1 {
    pub filing_id: &'static str,
    pub entity_bucket: u8,
}

/// Version 2 of a row of [`DATASET`]. It adds `unit`, which a row of version 1 reads as `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeRow {
    pub filing_id: String,
    pub entity_bucket: u8,
    pub unit: Option<String>,
}

fn fake_schema(version: u16) -> Option<SchemaRef> {
    let mut fields = vec![
        Field::new("filing_id", DataType::Utf8, false),
        Field::new("entity_bucket", DataType::UInt8, false),
    ];
    match version {
        1 => {}
        2 => fields.push(Field::new("unit", DataType::Utf8, true)),
        _ => return None,
    }
    Some(Arc::new(Schema::new(fields)))
}

fn fake_batch(version: u16, columns: Vec<ArrayRef>) -> Result<RecordBatch, ArrowRowError> {
    let schema =
        fake_schema(version).expect("Given version 1 or 2, the schema should always exist");
    RecordBatch::try_new(schema, columns).map_err(|error| ArrowRowError::FailedBatchCreation {
        dataset: DATASET,
        reason: error.to_string(),
    })
}

impl ArrowRow for FakeRowV1 {
    const DATASET: &'static str = DATASET;
    const SCHEMA_VERSION: u16 = 1;

    fn schema(version: u16) -> Option<SchemaRef> {
        fake_schema(version).filter(|_| version == 1)
    }

    fn to_batch(rows: &[Self]) -> Result<RecordBatch, ArrowRowError> {
        fake_batch(
            1,
            vec![
                Arc::new(StringArray::from_iter_values(
                    rows.iter().map(|row| row.filing_id),
                )),
                Arc::new(UInt8Array::from_iter_values(
                    rows.iter().map(|row| row.entity_bucket),
                )),
            ],
        )
    }

    fn from_batch(_batch: &RecordBatch, _version: u16) -> Result<Vec<Self>, ArrowRowError> {
        unimplemented!("The tests read the rows as `FakeRow`")
    }
}

impl ArrowRow for FakeRow {
    const DATASET: &'static str = DATASET;
    const SCHEMA_VERSION: u16 = 2;

    fn schema(version: u16) -> Option<SchemaRef> {
        fake_schema(version)
    }

    fn to_batch(rows: &[Self]) -> Result<RecordBatch, ArrowRowError> {
        fake_batch(
            2,
            vec![
                Arc::new(StringArray::from_iter_values(
                    rows.iter().map(|row| row.filing_id.as_str()),
                )),
                Arc::new(UInt8Array::from_iter_values(
                    rows.iter().map(|row| row.entity_bucket),
                )),
                Arc::new(
                    rows.iter()
                        .map(|row| row.unit.as_deref())
                        .collect::<StringArray>(),
                ),
            ],
        )
    }

    fn from_batch(batch: &RecordBatch, version: u16) -> Result<Vec<Self>, ArrowRowError> {
        let Some(schema) = fake_schema(version) else {
            return Err(ArrowRowError::UnknownSchemaVersion {
                dataset: DATASET,
                version,
            });
        };
        // Like a real row type, the fake rejects a batch of another version.
        if batch.schema().fields() != schema.fields() {
            return Err(ArrowRowError::UnexpectedSchema {
                dataset: DATASET,
                version,
                reason: "the fields differ".to_owned(),
            });
        }
        let text = |name: &str| {
            batch
                .column_by_name(name)
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        };
        let filing_ids =
            text("filing_id").expect("Given a fake part, the column should always exist");
        let entity_buckets = batch
            .column_by_name("entity_bucket")
            .and_then(|column| column.as_any().downcast_ref::<UInt8Array>())
            .expect("Given a fake part, the column should always exist");
        let units = text("unit");
        Ok((0..batch.num_rows())
            .map(|row| Self {
                filing_id: filing_ids.value(row).to_owned(),
                entity_bucket: entity_buckets.value(row),
                unit: units
                    .filter(|units| units.is_valid(row))
                    .map(|units| units.value(row).to_owned()),
            })
            .collect())
    }
}

impl PartOrder for FakeRowV1 {
    fn write_sort_key(&self, byte_form: &mut ByteForm) {
        byte_form.utf8(self.filing_id);
    }

    fn write_row_group_key(&self, byte_form: &mut ByteForm) {
        byte_form.integer(self.entity_bucket);
    }
}

impl PartOrder for FakeRow {
    fn write_sort_key(&self, byte_form: &mut ByteForm) {
        byte_form.utf8(&self.filing_id);
    }

    fn write_row_group_key(&self, byte_form: &mut ByteForm) {
        byte_form.integer(self.entity_bucket);
    }
}

/// Writes the rows as part 0 of the unit, and returns the unit with the part and its version.
pub async fn unit_with_part<R: ArrowRow + PartOrder>(
    store: &WriteOnceFilesystemRepository,
    unit: UnitManifestRow,
    rows: Vec<R>,
) -> UnitManifestRow {
    let location = PartLocation {
        source: unit.source,
        unit_key: unit.unit_key.clone(),
        input_digest: unit.input_digest.clone(),
        transform_version: unit.transform_version,
        execution_id: unit.execution_id,
        attempt: TaskAttempt {
            number: 0,
            is_bucket_split: false,
        },
    };
    let file = write_part(store, &location, 0, rows)
        .await
        .expect("Given an empty attempt folder, the part write should always succeed");
    UnitManifestRow {
        schema_version: R::SCHEMA_VERSION,
        row_count: file.row_count,
        files: vec![file],
        ..unit
    }
}

/// Returns a Parquet file with one column `filing_id` and no `entity_bucket`.
pub fn part_without_buckets() -> Bytes {
    let schema = Arc::new(Schema::new(vec![Field::new(
        "filing_id",
        DataType::Utf8,
        false,
    )]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(StringArray::from(vec!["0000320193-24-000006"]))],
    )
    .expect("Given a column of the schema, the batch should always build");
    let mut writer = ArrowWriter::try_new(Vec::new(), schema, None)
        .expect("Given a schema, the writer should always build");
    writer
        .write(&batch)
        .expect("Given a batch of the schema, the write should always succeed");
    Bytes::from(
        writer
            .into_inner()
            .expect("Given a written batch, the file should always close"),
    )
}
