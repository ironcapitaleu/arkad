//! # Unit Reader
//!
//! Provides the reads of the units of a dataset: [`current_units()`], the current unit of each
//! unit key, [`current_units_as_of()`], the units as the store held them at a time, and
//! [`series_units()`], the units of a series dataset.
//!
//! The reader finds units through the `unit_manifests` list files only. It never lists a folder of
//! a dataset, because a listing also finds the parts of replaced units and of failed attempts.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use bytes::Bytes;
use futures_util::TryStreamExt;
use jiff::Timestamp;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use storage::implementations::repository::filesystem::{DocumentKey, KeyPrefix};
use storage::implementations::repository::write_once::Blob;
use storage::{ListRepository, ReadRepository};
use uuid::Uuid;

use crate::datasets::implementations::UnitKind;
use crate::datasets::part_writer::constants::FOOTER_SCHEMA_VERSION;
use crate::datasets::{ArrowRow, SourceKind, UnitManifestRow, UnitReadError};

#[cfg(test)]
mod fixtures;

/// The current units of one dataset and source, by unit kind and `unit_key`.
pub type CurrentUnits = BTreeMap<(UnitKind, String), UnitManifestRow>;

/// Returns the current unit of each unit key of the dataset and the source.
///
/// Reads every list file below `unit_manifests/` and keeps the rows of the dataset whose source
/// equals `source`. Pass `None` for a dataset with no source, such as a series dataset. Per unit
/// kind and `unit_key`, the current unit is the row with the latest `written_at`. A unit without a
/// row in a list file does not exist for the reader.
///
/// # Errors
///
/// Returns:
/// - [`UnitReadError::FailedManifestListing`] if the store does not list the list files.
/// - [`UnitReadError::FailedManifestRead`] if the store does not read a list file.
/// - [`UnitReadError::MissingManifestFile`] if a listed list file is gone.
/// - [`UnitReadError::FailedManifestDecoding`] if a list file is not Parquet, or its footer holds
///   no valid `schema_version`.
/// - [`UnitReadError::FailedRowConversion`] if the batches of a list file do not turn into rows.
///
/// # Examples
///
/// ```rust
/// use sec::datasets::{SourceKind, current_units};
/// use storage::implementations::repository::write_once::WriteOnceFilesystemRepository;
///
/// let directory = tempfile::tempdir()
///     .expect("Given a writable temporary folder, the creation should always succeed");
/// let store = WriteOnceFilesystemRepository::new(directory.path());
///
/// let expected_result = 0;
///
/// let runtime = tokio::runtime::Runtime::new()
///     .expect("Given a fresh process, the creation of a runtime should always succeed");
/// let units = runtime
///     .block_on(current_units(&store, "filings", Some(SourceKind::Notes)))
///     .expect("Given an empty store, the read should always succeed");
/// let result = units.len();
///
/// assert_eq!(result, expected_result);
/// ```
pub async fn current_units<S>(
    store: &S,
    dataset: &str,
    source: Option<SourceKind>,
) -> Result<CurrentUnits, UnitReadError>
where
    S: ReadRepository<Key = DocumentKey, Record = Blob>
        + ListRepository<Key = DocumentKey, Filter = KeyPrefix>,
{
    current_units_as_of(store, dataset, source, Timestamp::MAX).await
}

/// Returns, per unit key of the dataset and the source, the latest unit written at or before
/// `as_of`.
///
/// This is the read of the dataset as the store held it at `as_of`. A unit written after `as_of`
/// does not exist for this read, and the unit it replaced stays current. The keys and the source
/// work as in [`current_units()`].
///
/// # Errors
///
/// Returns the errors of [`current_units()`].
///
/// # Examples
///
/// ```rust
/// use jiff::Timestamp;
/// use sec::datasets::{SourceKind, current_units_as_of};
/// use storage::implementations::repository::write_once::WriteOnceFilesystemRepository;
///
/// let directory = tempfile::tempdir()
///     .expect("Given a writable temporary folder, the creation should always succeed");
/// let store = WriteOnceFilesystemRepository::new(directory.path());
/// let as_of: Timestamp = "2026-11-28T12:00:00Z"
///     .parse()
///     .expect("Given a valid RFC 3339 text, the time should always parse");
///
/// let expected_result = 0;
///
/// let runtime = tokio::runtime::Runtime::new()
///     .expect("Given a fresh process, the creation of a runtime should always succeed");
/// let units = runtime
///     .block_on(current_units_as_of(&store, "filings", Some(SourceKind::Notes), as_of))
///     .expect("Given an empty store, the read should always succeed");
/// let result = units.len();
///
/// assert_eq!(result, expected_result);
/// ```
pub async fn current_units_as_of<S>(
    store: &S,
    dataset: &str,
    source: Option<SourceKind>,
    as_of: Timestamp,
) -> Result<CurrentUnits, UnitReadError>
where
    S: ReadRepository<Key = DocumentKey, Record = Blob>
        + ListRepository<Key = DocumentKey, Filter = KeyPrefix>,
{
    let mut units = CurrentUnits::new();
    for row in manifest_rows(store).await? {
        if row.dataset == dataset && row.source == source && row.written_at <= as_of {
            keep_latest(&mut units, (row.unit_kind, row.unit_key.clone()), row);
        }
    }
    Ok(units)
}

/// Returns the units of a series dataset that a read of its current rows takes.
///
/// A series run is the set of `series_base` and `series_delta` units of the dataset with one
/// `execution_id`. Per `unit_key` of a run, only the unit with the latest `written_at` enters, so
/// a retry of a task replaces its first attempt. A run counts when each of its units has a
/// `computed_at` and a `run_tasks` that equals the count of distinct unit keys of the run.
///
/// The units are those of the newest `series_base` run that counts, and of every `series_delta` run
/// that counts with a later `computed_at`. They sort by `computed_at`, then by `unit_key`. If no
/// `series_base` run counts, the list is empty.
///
/// # Errors
///
/// Returns the errors of [`current_units()`].
///
/// # Examples
///
/// ```rust
/// use sec::datasets::series_units;
/// use storage::implementations::repository::write_once::WriteOnceFilesystemRepository;
///
/// let directory = tempfile::tempdir()
///     .expect("Given a writable temporary folder, the creation should always succeed");
/// let store = WriteOnceFilesystemRepository::new(directory.path());
///
/// let expected_result = 0;
///
/// let runtime = tokio::runtime::Runtime::new()
///     .expect("Given a fresh process, the creation of a runtime should always succeed");
/// let units = runtime
///     .block_on(series_units(&store, "standardized_facts"))
///     .expect("Given an empty store, the read should always succeed");
/// let result = units.len();
///
/// assert_eq!(result, expected_result);
/// ```
pub async fn series_units<S>(
    store: &S,
    dataset: &str,
) -> Result<Vec<UnitManifestRow>, UnitReadError>
where
    S: ReadRepository<Key = DocumentKey, Record = Blob>
        + ListRepository<Key = DocumentKey, Filter = KeyPrefix>,
{
    let mut runs: BTreeMap<Uuid, BTreeMap<String, UnitManifestRow>> = BTreeMap::new();
    for row in manifest_rows(store).await? {
        let is_series = matches!(row.unit_kind, UnitKind::SeriesBase | UnitKind::SeriesDelta);
        if row.dataset == dataset && is_series {
            let run = runs.entry(row.execution_id).or_default();
            keep_latest(run, row.unit_key.clone(), row);
        }
    }
    let mut units: Vec<UnitManifestRow> = runs
        .into_values()
        .filter(run_counts)
        .flat_map(BTreeMap::into_values)
        .collect();
    let base_computed_at = units
        .iter()
        .filter(|unit| unit.unit_kind == UnitKind::SeriesBase)
        .filter_map(|unit| unit.computed_at)
        .max();
    let Some(base_computed_at) = base_computed_at else {
        return Ok(Vec::new());
    };
    units.retain(|unit| match unit.unit_kind {
        UnitKind::SeriesBase => unit.computed_at == Some(base_computed_at),
        _ => unit.computed_at > Some(base_computed_at),
    });
    units.sort_by(|left, right| {
        (left.computed_at, &left.unit_key).cmp(&(right.computed_at, &right.unit_key))
    });
    Ok(units)
}

/// Returns whether a series run counts: each unit has a `computed_at`, and a `run_tasks` that
/// equals the count of distinct unit keys of the run.
fn run_counts(run: &BTreeMap<String, UnitManifestRow>) -> bool {
    let unit_keys = u16::try_from(run.len()).ok();
    run.values().all(|unit| {
        unit.computed_at.is_some() && unit.run_tasks.is_some() && unit.run_tasks == unit_keys
    })
}

/// Puts the row under the key, unless the map holds a row of the key with a later `written_at`.
fn keep_latest<K: Ord>(units: &mut BTreeMap<K, UnitManifestRow>, key: K, row: UnitManifestRow) {
    match units.entry(key) {
        Entry::Vacant(entry) => {
            entry.insert(row);
        }
        Entry::Occupied(mut entry) => {
            if row.written_at > entry.get().written_at {
                entry.insert(row);
            }
        }
    }
}

/// Returns the rows of every list file below `unit_manifests/`.
async fn manifest_rows<S>(store: &S) -> Result<Vec<UnitManifestRow>, UnitReadError>
where
    S: ReadRepository<Key = DocumentKey, Record = Blob>
        + ListRepository<Key = DocumentKey, Filter = KeyPrefix>,
{
    let folder = DocumentKey::new(UnitManifestRow::DATASET)
        .expect("Given the fixed name of the dataset, the key should always build");
    let keys: Vec<DocumentKey> = store
        .list_keys(KeyPrefix::from(folder))
        .try_collect()
        .await
        .map_err(|source| UnitReadError::FailedManifestListing { source })?;
    let mut rows = Vec::new();
    for key in keys {
        let path = key.as_path().to_string_lossy().into_owned();
        let blob = store
            .get(key)
            .await
            .map_err(|source| UnitReadError::FailedManifestRead {
                path: path.clone(),
                source,
            })?
            .ok_or_else(|| UnitReadError::MissingManifestFile { path: path.clone() })?;
        rows.extend(decode_manifest(&path, blob.bytes)?);
    }
    Ok(rows)
}

/// Decodes the rows of one list file, with the schema version that its footer names.
fn decode_manifest(path: &str, bytes: Bytes) -> Result<Vec<UnitManifestRow>, UnitReadError> {
    let failed_decoding = |reason: String| UnitReadError::FailedManifestDecoding {
        path: path.to_owned(),
        reason,
    };
    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes)
        .map_err(|error| failed_decoding(error.to_string()))?;
    let version = builder
        .metadata()
        .file_metadata()
        .key_value_metadata()
        .and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry.key == FOOTER_SCHEMA_VERSION)
        })
        .and_then(|entry| entry.value.as_deref()?.parse::<u16>().ok())
        .ok_or_else(|| {
            failed_decoding(format!("the footer holds no valid {FOOTER_SCHEMA_VERSION}"))
        })?;
    let reader = builder
        .build()
        .map_err(|error| failed_decoding(error.to_string()))?;
    let mut rows = Vec::new();
    for batch in reader {
        let batch = batch.map_err(|error| failed_decoding(error.to_string()))?;
        rows.extend(
            UnitManifestRow::from_batch(&batch, version).map_err(|source| {
                UnitReadError::FailedRowConversion {
                    path: path.to_owned(),
                    source: Box::new(source),
                }
            })?,
        );
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use parquet::arrow::ArrowWriter;
    use pretty_assertions::assert_eq;

    use super::fixtures::{DATASET, notes_unit, series_unit, store, time, write_units};
    use super::*;

    const BASE: UnitKind = UnitKind::SeriesBase;
    const DELTA: UnitKind = UnitKind::SeriesDelta;

    /// Returns the `unit_key` of each unit of `series_units`.
    async fn series_keys<S>(store: &S) -> Vec<String>
    where
        S: ReadRepository<Key = DocumentKey, Record = Blob>
            + ListRepository<Key = DocumentKey, Filter = KeyPrefix>,
    {
        series_units(store, DATASET)
            .await
            .expect("Given list files that the writer wrote, the read should always succeed")
            .into_iter()
            .map(|unit| unit.unit_key)
            .collect()
    }

    #[tokio::test]
    async fn should_skip_a_later_unit_when_reading_as_of_a_time_before_it() {
        let (_directory, store) = store();
        write_units(
            &store,
            0,
            0,
            vec![notes_unit("2024q1", 1)],
            "2026-11-28T12:00:00Z",
        )
        .await;
        write_units(
            &store,
            0,
            0,
            vec![notes_unit("2024q1", 2)],
            "2026-11-28T13:00:00Z",
        )
        .await;
        let as_of = time("2026-11-28T12:30:00Z");

        let expected_result = vec![Uuid::from_u128(1)];

        let result: Vec<Uuid> =
            current_units_as_of(&store, DATASET, Some(SourceKind::Notes), as_of)
                .await
                .expect("Given list files that the writer wrote, the read should always succeed")
                .into_values()
                .map(|unit| unit.execution_id)
                .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_the_unit_written_at_the_time_when_reading_as_of_that_time() {
        let (_directory, store) = store();
        write_units(
            &store,
            0,
            0,
            vec![notes_unit("2024q1", 1)],
            "2026-11-28T12:00:00Z",
        )
        .await;
        write_units(
            &store,
            0,
            0,
            vec![notes_unit("2024q1", 2)],
            "2026-11-28T13:00:00Z",
        )
        .await;
        let as_of = time("2026-11-28T13:00:00Z");

        let expected_result = vec![Uuid::from_u128(2)];

        let result: Vec<Uuid> =
            current_units_as_of(&store, DATASET, Some(SourceKind::Notes), as_of)
                .await
                .expect("Given list files that the writer wrote, the read should always succeed")
                .into_values()
                .map(|unit| unit.execution_id)
                .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_no_series_unit_when_the_run_does_not_count() {
        let (_directory, store) = store();
        // The run has 2 tasks, but only task 0 wrote its unit.
        let unit = series_unit(BASE, "base-t0", 1, "2026-11-28T06:00:00Z", 2);
        write_units(&store, 0, 0, vec![unit], "2026-11-28T06:10:00Z").await;

        let expected_result: Vec<String> = Vec::new();

        let result = series_keys(&store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_newest_base_and_later_deltas_when_their_runs_count() {
        let (_directory, store) = store();
        let runs = [
            (
                series_unit(BASE, "base-t0", 1, "2026-11-01T06:00:00Z", 1),
                "2026-11-01T07:00:00Z",
            ),
            (
                series_unit(
                    DELTA,
                    "2026-11-02T060000.000Z-t0",
                    2,
                    "2026-11-02T06:00:00Z",
                    1,
                ),
                "2026-11-02T07:00:00Z",
            ),
            (
                series_unit(
                    BASE,
                    "base-2026-11-03T060000.000Z-t0",
                    3,
                    "2026-11-03T06:00:00Z",
                    1,
                ),
                "2026-11-03T07:00:00Z",
            ),
            (
                series_unit(
                    DELTA,
                    "2026-11-04T060000.000Z-t0",
                    4,
                    "2026-11-04T06:00:00Z",
                    1,
                ),
                "2026-11-04T07:00:00Z",
            ),
            // A run of 2 tasks with one unit does not count.
            (
                series_unit(
                    DELTA,
                    "2026-11-05T060000.000Z-t0",
                    5,
                    "2026-11-05T06:00:00Z",
                    2,
                ),
                "2026-11-05T07:00:00Z",
            ),
        ];
        for (unit, now) in runs {
            write_units(&store, 0, 0, vec![unit], now).await;
        }

        let expected_result = vec![
            "base-2026-11-03T060000.000Z-t0".to_owned(),
            "2026-11-04T060000.000Z-t0".to_owned(),
        ];

        let result = series_keys(&store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_no_series_unit_when_only_delta_runs_count() {
        let (_directory, store) = store();
        let unit = series_unit(
            DELTA,
            "2026-11-02T060000.000Z-t0",
            1,
            "2026-11-02T06:00:00Z",
            1,
        );
        write_units(&store, 0, 0, vec![unit], "2026-11-02T07:00:00Z").await;

        let expected_result: Vec<String> = Vec::new();

        let result = series_keys(&store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_take_the_latest_attempt_when_a_task_of_a_run_wrote_its_unit_twice() {
        let (_directory, store) = store();
        let unit = series_unit(BASE, "base-t0", 1, "2026-11-01T06:00:00Z", 1);
        write_units(&store, 0, 0, vec![unit.clone()], "2026-11-01T07:00:00Z").await;
        write_units(&store, 0, 1, vec![unit], "2026-11-01T08:00:00Z").await;

        let expected_result = vec![time("2026-11-01T08:00:00Z")];

        let result: Vec<Timestamp> = series_units(&store, DATASET)
            .await
            .expect("Given list files that the writer wrote, the read should always succeed")
            .into_iter()
            .map(|unit| unit.written_at)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_count_a_run_when_each_task_wrote_its_unit_in_its_own_list_file() {
        let (_directory, store) = store();
        let first = series_unit(BASE, "base-t0", 1, "2026-11-01T06:00:00Z", 2);
        let second = series_unit(BASE, "base-t1", 1, "2026-11-01T06:00:00Z", 2);
        write_units(&store, 0, 0, vec![first], "2026-11-01T07:00:00Z").await;
        write_units(&store, 1, 0, vec![second], "2026-11-01T07:05:00Z").await;

        let expected_result = vec!["base-t0".to_owned(), "base-t1".to_owned()];

        let result = series_keys(&store).await;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_manifest_decoding_when_the_footer_has_no_schema_version() {
        let batch =
            UnitManifestRow::to_batch(&[]).expect("Given no rows, the batch should always build");
        let mut writer = ArrowWriter::try_new(Vec::new(), batch.schema(), None)
            .expect("Given the schema of a batch, the writer should always build");
        writer
            .write(&batch)
            .expect("Given a batch of the schema, the write should always succeed");
        let bytes = Bytes::from(
            writer
                .into_inner()
                .expect("Given a written batch, the file should always close"),
        );

        let expected_result = Err(UnitReadError::FailedManifestDecoding {
            path: "unit_manifests/execution=e/task=0-0.parquet".to_owned(),
            reason: "the footer holds no valid schema_version".to_owned(),
        });

        let result = decode_manifest("unit_manifests/execution=e/task=0-0.parquet", bytes);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_manifest_decoding_when_the_bytes_are_not_parquet() {
        let bytes = Bytes::from_static(b"not parquet");

        let expected_result = Err(UnitReadError::FailedManifestDecoding {
            path: "unit_manifests/execution=e/task=0-0.parquet".to_owned(),
            reason: "Parquet error: Invalid Parquet file. Corrupt footer".to_owned(),
        });

        let result = decode_manifest("unit_manifests/execution=e/task=0-0.parquet", bytes);

        assert_eq!(result, expected_result);
    }
}
