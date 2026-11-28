//! # Fixtures
//!
//! Provides the store, the times, and the `unit_manifests` rows of the tests of the reader.

use jiff::Timestamp;
use storage::implementations::repository::filesystem::IngestionMode;
use storage::implementations::repository::write_once::WriteOnceFilesystemRepository;
use storage::shared::Sha256Digest;
use tempfile::TempDir;
use uuid::Uuid;

use crate::datasets::implementations::UnitKind;
use crate::datasets::{
    SourceKind, TaskAttempt, TaskLocation, UnitManifestRow, WrittenUnit, write_task_manifest,
};

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
