//! # Task Manifest Writer
//!
//! Provides [`write_task_manifest()`], which writes the `unit_manifests` rows of the units of one
//! task in one list file, and [`TaskLocation`], the task that the path of the file names.
//!
//! The list file is the last write of a task (rule 4 of a write). Its rows make the units of the
//! task visible together, or not at all.

use std::cmp;

use jiff::Timestamp;
use parquet::arrow::ArrowWriter;
use parquet::errors::ParquetError;
use parquet::file::metadata::KeyValue;
use storage::WriteRepository;
use storage::implementations::repository::filesystem::DocumentKey;
use storage::implementations::repository::write_once::Blob;
use uuid::Uuid;

use crate::datasets::part_writer::constants::{
    FOOTER_DATASET, FOOTER_SCHEMA_VERSION, ROW_GROUP_ROWS,
};
use crate::datasets::part_writer::parquet_properties;
use crate::datasets::{ArrowRow, TaskAttempt, UnitManifestRow, UnitWriteError, WrittenUnit};

/// The task attempt that the path of its list file names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskLocation {
    /// The job that runs the task.
    pub execution_id: Uuid,
    /// The index of the task in the job, from 0.
    pub task_index: u32,
    /// The attempt of the task.
    pub attempt: TaskAttempt,
}

/// Writes the `unit_manifests` rows of the units of one task in one list file, once.
///
/// The file goes to `unit_manifests/execution=<execution_id>/task=<n>-<attempt>.parquet`. Each
/// row gets the `written_at` of the later of `now` and the `written_at` of the current unit of its
/// key plus 1 ms, at millisecond precision. So `written_at` rises with each unit of a key, also
/// on two writes in one millisecond. If `units` is empty, the function writes nothing.
///
/// # Errors
///
/// Returns:
/// - [`UnitWriteError::InvalidTaskLocation`] if a unit names another execution or attempt than
///   `task`.
/// - [`UnitWriteError::FailedManifestConversion`] if the rows do not turn into a record batch.
/// - [`UnitWriteError::FailedManifestEncoding`] if Parquet rejects the batch.
/// - [`UnitWriteError::FailedManifestWrite`] if the store does not write the file, for example
///   because a blob exists at its key.
pub async fn write_task_manifest<S>(
    store: &S,
    task: &TaskLocation,
    units: Vec<WrittenUnit>,
    now: Timestamp,
) -> Result<(), UnitWriteError>
where
    S: WriteRepository<Record = Blob>,
{
    if units.is_empty() {
        return Ok(());
    }
    if let Some(unit) = units
        .iter()
        .find(|unit| unit.row.execution_id != task.execution_id || unit.attempt != task.attempt)
    {
        return Err(UnitWriteError::InvalidTaskLocation {
            unit_key: unit.row.unit_key.clone(),
            execution_id: unit.row.execution_id,
            attempt: unit.attempt,
            task_execution_id: task.execution_id,
            task_attempt: task.attempt,
        });
    }
    let rows: Vec<UnitManifestRow> = units
        .into_iter()
        .map(|unit| UnitManifestRow {
            written_at: written_at(now, unit.current_written_at),
            ..unit.row
        })
        .collect();
    let bytes = encode_manifest(&rows)?;
    store
        .persist(Blob::new(task_key(task), bytes))
        .await
        .map_err(|source| UnitWriteError::FailedManifestWrite { source })
}

/// Returns the later of `now` and `current` plus 1 ms, cut to whole milliseconds.
fn written_at(now: Timestamp, current: Option<Timestamp>) -> Timestamp {
    let now = now.as_millisecond();
    let millisecond = current.map_or(now, |current| {
        cmp::max(now, current.as_millisecond().saturating_add(1))
    });
    // Past the last time that `Timestamp` holds, the unit keeps that last time.
    Timestamp::from_millisecond(millisecond).unwrap_or(Timestamp::MAX)
}

/// Returns the key of the list file of the task.
fn task_key(task: &TaskLocation) -> DocumentKey {
    let path = format!(
        "{}/execution={}/task={}-{}.parquet",
        UnitManifestRow::DATASET,
        task.execution_id,
        task.task_index,
        task.attempt,
    );
    DocumentKey::new(path)
        .expect("Given a UUID, a task index, and an attempt, the path should always form a key")
}

/// Encodes the rows as one Parquet file, with zstd and the footer entries `dataset` and
/// `schema_version`.
fn encode_manifest(rows: &[UnitManifestRow]) -> Result<Vec<u8>, UnitWriteError> {
    let batch = UnitManifestRow::to_batch(rows)?;
    let footer = vec![
        KeyValue::new(
            FOOTER_DATASET.to_owned(),
            UnitManifestRow::DATASET.to_owned(),
        ),
        KeyValue::new(
            FOOTER_SCHEMA_VERSION.to_owned(),
            UnitManifestRow::SCHEMA_VERSION.to_string(),
        ),
    ];
    let failed_encoding = |error: ParquetError| UnitWriteError::FailedManifestEncoding {
        reason: error.to_string(),
    };
    let properties = parquet_properties(footer, ROW_GROUP_ROWS).map_err(failed_encoding)?;
    let mut writer = ArrowWriter::try_new(Vec::new(), batch.schema(), Some(properties))
        .map_err(failed_encoding)?;
    writer.write(&batch).map_err(failed_encoding)?;
    writer.into_inner().map_err(failed_encoding)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn time(text: &str) -> Timestamp {
        text.parse()
            .expect("Given a valid RFC 3339 text, the time should always parse")
    }

    #[test]
    fn should_return_now_when_the_key_has_no_current_unit() {
        let now = time("2026-11-28T12:00:00.250Z");

        let expected_result = time("2026-11-28T12:00:00.250Z");

        let result = written_at(now, None);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_current_unit_plus_1_ms_when_now_is_not_later() {
        let now = time("2026-11-28T12:00:00.250Z");
        let current = Some(time("2026-11-28T12:00:00.250Z"));

        let expected_result = time("2026-11-28T12:00:00.251Z");

        let result = written_at(now, current);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cut_now_to_the_millisecond_when_now_is_later_by_less_than_1_ms() {
        let now = time("2026-11-28T12:00:00.250900Z");
        let current = Some(time("2026-11-28T12:00:00.250Z"));

        let expected_result = time("2026-11-28T12:00:00.251Z");

        let result = written_at(now, current);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_now_when_now_is_later_than_the_current_unit() {
        let now = time("2026-11-28T12:00:01Z");
        let current = Some(time("2026-11-28T12:00:00.250Z"));

        let expected_result = time("2026-11-28T12:00:01Z");

        let result = written_at(now, current);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_execution_task_and_attempt_in_the_key_of_the_list_file() {
        let task = TaskLocation {
            execution_id: Uuid::nil(),
            task_index: 3,
            attempt: TaskAttempt {
                number: 1,
                is_bucket_split: true,
            },
        };

        let expected_result =
            "unit_manifests/execution=00000000-0000-0000-0000-000000000000/task=3-1-b.parquet";

        let key = task_key(&task);
        let result = key.as_path().to_string_lossy().into_owned();

        assert_eq!(result, expected_result);
    }
}
