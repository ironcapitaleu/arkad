//! # Unit Writer
//!
//! Provides the [`UnitWriter`], which writes the parts of one unit of a dataset, and
//! [`input_digest()`], the digest of the inputs of a unit.
//!
//! The writer follows rules 1 to 3 of a write. [`UnitWriter::start`] computes the `input_digest`
//! (rule 1), and skips the unit if the current unit of its key has the same digest and
//! `transform_version` (rule 2). [`UnitWriter::push_groups`] collects the rows of whole digest
//! groups and writes a part through [`write_part()`] when the rows reach [`PART_FLUSH_BYTES`]
//! (rule 3). [`UnitWriter::finish`] writes the last part and computes the `content_digest`. Its
//! [`WrittenUnit`] stays invisible until
//! [`write_task_manifest()`](crate::datasets::write_task_manifest()) writes the list file of the
//! task (rule 4).
//!
//! ## Modules
//!
//! - [`constants`]: The size at which the writer flushes a part.

use std::mem;

use jiff::Timestamp;
use storage::implementations::repository::filesystem::{DocumentKey, IngestionMode, KeyPrefix};
use storage::implementations::repository::write_once::Blob;
use storage::shared::Sha256Digest;
use storage::{ListRepository, ReadRepository, WriteRepository};
use uuid::Uuid;

use self::constants::PART_FLUSH_BYTES;
use crate::datasets::implementations::{
    DroppedRows, NotesSplit, TableRows, UnitFile, UnitInput, UnitKind,
};
use crate::datasets::{
    ArrowRow, PartLocation, PartOrder, SourceKind, TaskAttempt, UnitManifestRow, UnitWriteError,
    current_units, write_part,
};
use crate::shared::row_digest::{
    ByteForm, DigestRow, GroupHash, content_digest, group_hashes_and_row_bytes,
};

pub mod constants;

/// What a unit is and what it read: the values the caller knows before the first row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitSpec {
    /// The source of the unit. `None` for the series datasets.
    pub source: Option<SourceKind>,
    /// The kind of the unit.
    pub unit_kind: UnitKind,
    /// The key of the unit, for example `2024q1`.
    pub unit_key: String,
    /// Each raw document or dataset file that the unit reads.
    pub inputs: Vec<UnitInput>,
    /// The version of the transform output.
    pub transform_version: u32,
    /// How the job splits a Notes file. `None` for the other units.
    pub split: Option<NotesSplit>,
    /// The start time of the series run of a series unit. `None` for the other units.
    pub computed_at: Option<Timestamp>,
    /// The task count of the series run of a series unit. `None` for the other units.
    pub run_tasks: Option<u16>,
    /// The job that writes the unit.
    pub execution_id: Uuid,
    /// The attempt of the task that writes the unit.
    pub attempt: TaskAttempt,
    /// The ingestion mode of the job.
    pub ingestion_mode: IngestionMode,
}

/// The result of [`UnitWriter::start`]: a skip by rule 2, or a writer of the unit.
#[derive(Debug)]
pub enum UnitStart<'s, R, S> {
    /// The current unit of the key has the same `input_digest` and `transform_version`.
    Skipped(UnitManifestRow),
    /// The writer of the new unit.
    Write(UnitWriter<'s, R, S>),
}

/// A unit whose parts the writer wrote, and whose row waits for the list file of its task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenUnit {
    /// The row of the unit. [`write_task_manifest`](crate::datasets::write_task_manifest()) sets
    /// its `written_at`.
    pub(crate) row: UnitManifestRow,
    /// The `written_at` of the current unit of the key, if the key has one.
    pub(crate) current_written_at: Option<Timestamp>,
    /// The attempt of the task that wrote the parts.
    pub(crate) attempt: TaskAttempt,
}

/// The writer of the parts of one unit of the dataset of `R`.
///
/// The writer never holds a whole unit. It holds the rows of one part, and one [`GroupHash`] per
/// digest group.
#[derive(Debug)]
pub struct UnitWriter<'s, R, S> {
    store: &'s S,
    spec: UnitSpec,
    location: PartLocation,
    current_written_at: Option<Timestamp>,
    rows: Vec<R>,
    row_bytes: usize,
    flush_bytes: usize,
    files: Vec<UnitFile>,
    groups: Vec<GroupHash>,
}

impl<'s, R, S> UnitWriter<'s, R, S>
where
    R: ArrowRow + PartOrder + DigestRow,
    S: WriteRepository<Record = Blob>
        + ReadRepository<Key = DocumentKey, Record = Blob>
        + ListRepository<Key = DocumentKey, Filter = KeyPrefix>,
{
    /// Applies rules 1 and 2 of a write: computes the `input_digest` of the unit, and reads the
    /// current unit of its key through [`current_units()`].
    ///
    /// # Returns
    ///
    /// [`UnitStart::Skipped`] with the current unit if it has the same `input_digest` and
    /// `transform_version`. Otherwise [`UnitStart::Write`] with a writer of the new unit. An older
    /// unit with the same digest and version does not count.
    ///
    /// # Errors
    ///
    /// Returns [`UnitWriteError::FailedCurrentUnitRead`] if the writer cannot read the list files.
    pub async fn start(
        store: &'s S,
        spec: UnitSpec,
    ) -> Result<UnitStart<'s, R, S>, UnitWriteError> {
        let digest = input_digest(&spec.inputs);
        let current = current_units(store, R::DATASET, spec.source)
            .await?
            .remove(&(spec.unit_kind, spec.unit_key.clone()));
        let current_written_at = match current {
            Some(current)
                if current.input_digest == digest
                    && current.transform_version == spec.transform_version =>
            {
                return Ok(UnitStart::Skipped(current));
            }
            current => current.map(|current| current.written_at),
        };
        let location = PartLocation {
            source: spec.source,
            unit_key: spec.unit_key.clone(),
            input_digest: digest,
            transform_version: spec.transform_version,
            execution_id: spec.execution_id,
            attempt: spec.attempt,
        };
        Ok(UnitStart::Write(Self {
            store,
            spec,
            location,
            current_written_at,
            rows: Vec::new(),
            row_bytes: 0,
            flush_bytes: PART_FLUSH_BYTES,
            files: Vec::new(),
            groups: Vec::new(),
        }))
    }

    /// Adds the rows of whole digest groups, and writes a part when the rows reach
    /// [`PART_FLUSH_BYTES`].
    ///
    /// Each digest group must be whole in one call. The writer hashes the groups at once and keeps
    /// only their [`GroupHash`]. The size of the rows is the length of their [`ByteForm`]. A part
    /// flushes after a call, so a part ends on a group boundary.
    ///
    /// # Errors
    ///
    /// Returns [`UnitWriteError::FailedPartWrite`] if [`write_part()`] does not write the part.
    pub async fn push_groups(&mut self, rows: Vec<R>) -> Result<(), UnitWriteError> {
        let (groups, row_bytes) = group_hashes_and_row_bytes(&rows);
        self.groups.extend(groups);
        self.row_bytes += row_bytes;
        self.rows.extend(rows);
        if self.row_bytes >= self.flush_bytes {
            self.flush().await?;
        }
        Ok(())
    }

    /// Writes the last part and computes the `content_digest` of the unit.
    ///
    /// The `source_rows` and `dropped_rows` are the counts of the unit, which the caller knows
    /// after the last row. A unit with no rows has no part.
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`UnitWriteError::FailedPartWrite`] if [`write_part()`] does not write the last part.
    /// - [`UnitWriteError::SplitDigestGroup`] if two calls of [`push_groups`](Self::push_groups)
    ///   held rows of one digest group.
    pub async fn finish(
        mut self,
        source_rows: Vec<TableRows>,
        dropped_rows: Vec<DroppedRows>,
    ) -> Result<WrittenUnit, UnitWriteError> {
        if !self.rows.is_empty() {
            self.flush().await?;
        }
        let content_digest = content_digest(self.groups)?;
        let spec = self.spec;
        let row = UnitManifestRow {
            dataset: R::DATASET.to_owned(),
            schema_version: R::SCHEMA_VERSION,
            source: spec.source,
            unit_kind: spec.unit_kind,
            unit_key: spec.unit_key,
            input_digest: self.location.input_digest,
            inputs: spec.inputs,
            transform_version: spec.transform_version,
            row_count: self.files.iter().map(|file| file.row_count).sum(),
            files: self.files,
            source_rows,
            split: spec.split,
            dropped_rows,
            content_digest,
            computed_at: spec.computed_at,
            run_tasks: spec.run_tasks,
            execution_id: spec.execution_id,
            ingestion_mode: spec.ingestion_mode,
            // The list file of the task sets the time of the write.
            written_at: Timestamp::UNIX_EPOCH,
        };
        Ok(WrittenUnit {
            row,
            current_written_at: self.current_written_at,
            attempt: spec.attempt,
        })
    }

    /// Writes the held rows as the next part, and drops them.
    async fn flush(&mut self) -> Result<(), UnitWriteError> {
        let part_number = u32::try_from(self.files.len())
            .expect("Given parts of about 256 MB, a unit never holds more than u32::MAX parts");
        let rows = mem::take(&mut self.rows);
        let file = write_part(self.store, &self.location, part_number, rows).await?;
        self.files.push(file);
        self.row_bytes = 0;
        Ok(())
    }
}

/// Returns the `input_digest` of a unit: the SHA-256 over its inputs, sorted by key and SHA-256.
///
/// The inputs enter in the byte form of a list of structs: each input is a struct of its key and
/// the hex text of its SHA-256. So the order of the inputs does not change the digest.
///
/// # Examples
///
/// ```rust
/// use sec::datasets::input_digest;
///
/// let inputs = Vec::new();
///
/// // The byte form of an empty list: marker 01 and the count 00000000.
/// let expected_result = "957b88b12730e646e0f33d3618b77dfa579e8231e3c59c7104be7165611c8027";
///
/// let digest = input_digest(&inputs);
/// let result = digest.as_str();
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn input_digest(inputs: &[UnitInput]) -> Sha256Digest {
    let mut pairs: Vec<(&str, &str)> = inputs
        .iter()
        .map(|input| (input.key.as_str(), input.sha256.as_str()))
        .collect();
    pairs.sort_unstable();
    let mut byte_form = ByteForm::new();
    byte_form.list(pairs.len());
    for (key, sha256) in pairs {
        byte_form.struct_(2).utf8(key).utf8(sha256);
    }
    Sha256Digest::of(byte_form.as_bytes())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow::array::{RecordBatch, StringArray};
    use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
    use futures_util::TryStreamExt;
    use pretty_assertions::assert_eq;
    use storage::implementations::repository::write_once::WriteOnceFilesystemRepository;
    use tempfile::TempDir;

    use super::*;
    use crate::datasets::{ArrowRowError, CurrentUnits, TaskLocation, write_task_manifest};
    use crate::shared::row_digest::DuplicateDigestGroup;

    /// A fact of a test dataset: one digest group per filing.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct FakeRow {
        filing_id: &'static str,
        concept: &'static str,
    }

    impl ArrowRow for FakeRow {
        const DATASET: &'static str = "fake_rows";
        const SCHEMA_VERSION: u16 = 1;

        fn schema(version: u16) -> Option<SchemaRef> {
            (version == 1).then(|| {
                Arc::new(Schema::new(vec![
                    Field::new("filing_id", DataType::Utf8, false),
                    Field::new("concept", DataType::Utf8, false),
                ]))
            })
        }

        fn to_batch(rows: &[Self]) -> Result<RecordBatch, ArrowRowError> {
            let text = |value: fn(&Self) -> &'static str| {
                Arc::new(StringArray::from_iter_values(rows.iter().map(value))) as _
            };
            let schema = Self::schema(1).expect("Given version 1, the schema should always exist");
            RecordBatch::try_new(
                schema,
                vec![text(|row| row.filing_id), text(|row| row.concept)],
            )
            .map_err(|error| ArrowRowError::FailedBatchCreation {
                dataset: Self::DATASET,
                reason: error.to_string(),
            })
        }

        fn from_batch(_batch: &RecordBatch, _version: u16) -> Result<Vec<Self>, ArrowRowError> {
            unimplemented!("The tests read the rows of `unit_manifests` only")
        }
    }

    impl PartOrder for FakeRow {
        fn write_sort_key(&self, byte_form: &mut ByteForm) {
            byte_form.utf8(self.filing_id).utf8(self.concept);
        }
    }

    impl DigestRow for FakeRow {
        fn write_group_id(&self, byte_form: &mut ByteForm) {
            byte_form.utf8(self.filing_id);
        }

        fn write_row_key(&self, byte_form: &mut ByteForm) {
            byte_form.utf8(self.concept);
        }

        fn write_row(&self, byte_form: &mut ByteForm) {
            byte_form.utf8(self.filing_id).utf8(self.concept);
        }
    }

    type FakeWriter<'s> = UnitWriter<'s, FakeRow, WriteOnceFilesystemRepository>;

    const FILING_A: &str = "0000320193-24-000006";
    const FILING_B: &str = "0000320193-24-000007";

    const fn row(filing_id: &'static str, concept: &'static str) -> FakeRow {
        FakeRow { filing_id, concept }
    }

    fn store() -> (TempDir, WriteOnceFilesystemRepository) {
        let directory = tempfile::tempdir()
            .expect("Given a writable temporary folder, the creation should always succeed");
        let store = WriteOnceFilesystemRepository::new(directory.path());
        (directory, store)
    }

    fn time(text: &str) -> Timestamp {
        text.parse()
            .expect("Given a valid RFC 3339 text, the time should always parse")
    }

    fn input(key: &str, content: &str) -> UnitInput {
        UnitInput {
            key: key.to_owned(),
            sha256: Sha256Digest::of(content),
            requested_url: None,
            produced_at: None,
        }
    }

    fn spec(transform_version: u32, execution: u128) -> UnitSpec {
        UnitSpec {
            source: Some(SourceKind::Notes),
            unit_kind: UnitKind::NotesFile,
            unit_key: "2024q1".to_owned(),
            inputs: vec![input("sec/notes/2024q1_notes.zip", "zip")],
            transform_version,
            split: Some(NotesSplit::Stream),
            computed_at: None,
            run_tasks: None,
            execution_id: Uuid::from_u128(execution),
            attempt: TaskAttempt {
                number: 0,
                is_bucket_split: false,
            },
            ingestion_mode: IngestionMode::Backfill,
        }
    }

    async fn writer(store: &WriteOnceFilesystemRepository, spec: UnitSpec) -> FakeWriter<'_> {
        match FakeWriter::start(store, spec).await {
            Ok(UnitStart::Write(writer)) => writer,
            other => panic!(
                "Given no current unit of the same version, the writer should start, got {other:?}"
            ),
        }
    }

    /// Writes the parts of one unit with two filings, without its list file.
    async fn written_unit(store: &WriteOnceFilesystemRepository, spec: UnitSpec) -> WrittenUnit {
        let mut writer = writer(store, spec).await;
        writer
            .push_groups(vec![row(FILING_A, "Assets"), row(FILING_A, "Liabilities")])
            .await
            .expect("Given an empty attempt folder, the part write should always succeed");
        writer
            .push_groups(vec![row(FILING_B, "Assets")])
            .await
            .expect("Given an empty attempt folder, the part write should always succeed");
        writer
            .finish(Vec::new(), Vec::new())
            .await
            .expect("Given whole digest groups, the unit should always finish")
    }

    /// Writes one unit and the list file of its task, with the task index 0.
    async fn write_unit(store: &WriteOnceFilesystemRepository, spec: UnitSpec, now: Timestamp) {
        let task = TaskLocation {
            execution_id: spec.execution_id,
            task_index: 0,
            attempt: spec.attempt,
        };
        let unit = written_unit(store, spec).await;
        write_task_manifest(store, &task, vec![unit], now)
            .await
            .expect("Given a new execution, the list file should always be new");
    }

    async fn current(store: &WriteOnceFilesystemRepository) -> CurrentUnits {
        current_units(store, FakeRow::DATASET, Some(SourceKind::Notes))
            .await
            .expect("Given list files that the writer wrote, the read should always succeed")
    }

    #[tokio::test]
    async fn should_skip_the_unit_when_a_rerun_has_the_same_inputs_and_version() {
        let (_directory, store) = store();
        write_unit(&store, spec(1, 1), time("2026-11-28T12:00:00Z")).await;
        let rerun = spec(1, 2);

        let expected_result = Some(Uuid::from_u128(1));

        let start = FakeWriter::start(&store, rerun)
            .await
            .expect("Given list files that the writer wrote, the start should always succeed");
        let result = match start {
            UnitStart::Skipped(current) => Some(current.execution_id),
            UnitStart::Write(_) => None,
        };

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_make_the_new_unit_current_when_the_transform_version_changes() {
        let (_directory, store) = store();
        write_unit(&store, spec(1, 1), time("2026-11-28T12:00:00Z")).await;
        write_unit(&store, spec(2, 2), time("2026-11-28T13:00:00Z")).await;

        let expected_result = vec![((UnitKind::NotesFile, "2024q1".to_owned()), 2)];

        let result: Vec<((UnitKind, String), u32)> = current(&store)
            .await
            .into_iter()
            .map(|(key, unit)| (key, unit.transform_version))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_latest_written_at_when_the_newer_unit_is_listed_first() {
        let (_directory, store) = store();
        // The list file of execution 2 sorts after that of execution 1, but holds the older unit.
        write_unit(&store, spec(1, 2), time("2026-11-28T12:00:00Z")).await;
        write_unit(&store, spec(2, 1), time("2026-11-28T13:00:00Z")).await;

        let expected_result = vec![2];

        let result: Vec<u32> = current(&store)
            .await
            .into_values()
            .map(|unit| unit.transform_version)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_start_a_write_when_the_inputs_change_at_the_same_version() {
        let (_directory, store) = store();
        write_unit(&store, spec(1, 1), time("2026-11-28T12:00:00Z")).await;
        let mut changed = spec(1, 2);
        changed.inputs = vec![input("sec/notes/2024q1_notes.zip", "corrected zip")];

        let expected_result = "write";

        let start = FakeWriter::start(&store, changed)
            .await
            .expect("Given list files that the writer wrote, the start should always succeed");
        let result = match start {
            UnitStart::Skipped(_) => "skip",
            UnitStart::Write(_) => "write",
        };

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_no_unit_of_another_source_when_reading_the_current_units() {
        let (_directory, store) = store();
        write_unit(&store, spec(1, 1), time("2026-11-28T12:00:00Z")).await;

        let expected_result = CurrentUnits::new();

        let result = current_units(&store, FakeRow::DATASET, Some(SourceKind::Instance))
            .await
            .expect("Given list files that the writer wrote, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_no_list_file_when_the_task_has_no_unit() {
        let (_directory, store) = store();
        let task = TaskLocation {
            execution_id: Uuid::from_u128(1),
            task_index: 0,
            attempt: spec(1, 1).attempt,
        };

        let expected_result: Vec<DocumentKey> = Vec::new();

        write_task_manifest(&store, &task, Vec::new(), time("2026-11-28T12:00:00Z"))
            .await
            .expect("Given no unit, the call should always succeed");
        let result: Vec<DocumentKey> = store
            .list_keys(KeyPrefix::root())
            .try_collect()
            .await
            .expect("Given a readable root, the listing should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_task_location_when_a_unit_names_another_execution() {
        let (_directory, store) = store();
        let unit = written_unit(&store, spec(1, 1)).await;
        let task = TaskLocation {
            execution_id: Uuid::from_u128(2),
            task_index: 0,
            attempt: unit.attempt,
        };

        let expected_result = Err(UnitWriteError::InvalidTaskLocation {
            unit_key: "2024q1".to_owned(),
            execution_id: Uuid::from_u128(1),
            attempt: unit.attempt,
            task_execution_id: Uuid::from_u128(2),
            task_attempt: unit.attempt,
        });

        let result =
            write_task_manifest(&store, &task, vec![unit], time("2026-11-28T12:00:00Z")).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_task_location_when_a_unit_names_another_attempt() {
        let (_directory, store) = store();
        let unit = written_unit(&store, spec(1, 1)).await;
        let retry = TaskAttempt {
            number: 1,
            is_bucket_split: false,
        };
        let task = TaskLocation {
            execution_id: Uuid::from_u128(1),
            task_index: 0,
            attempt: retry,
        };

        let expected_result = Err(UnitWriteError::InvalidTaskLocation {
            unit_key: "2024q1".to_owned(),
            execution_id: Uuid::from_u128(1),
            attempt: unit.attempt,
            task_execution_id: Uuid::from_u128(1),
            task_attempt: retry,
        });

        let result =
            write_task_manifest(&store, &task, vec![unit], time("2026-11-28T12:00:00Z")).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_no_visible_unit_when_the_task_stops_before_its_list_file() {
        let (_directory, store) = store();
        written_unit(&store, spec(1, 1)).await;

        let expected_result = CurrentUnits::new();

        let result = current(&store).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_raise_written_at_by_1_ms_when_two_writes_fall_in_one_millisecond() {
        let (_directory, store) = store();
        let now = time("2026-11-28T12:00:00.250Z");
        write_unit(&store, spec(1, 1), now).await;
        write_unit(&store, spec(2, 2), now).await;

        let expected_result = vec![time("2026-11-28T12:00:00.251Z")];

        let result: Vec<Timestamp> = current(&store)
            .await
            .into_values()
            .map(|unit| unit.written_at)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_a_part_per_flush_when_the_rows_reach_the_flush_size() {
        let (_directory, store) = store();
        let mut writer = writer(&store, spec(1, 1)).await;
        writer.flush_bytes = 1;

        let expected_result = vec![2, 1];

        let unit = {
            writer
                .push_groups(vec![row(FILING_A, "Assets"), row(FILING_A, "Liabilities")])
                .await
                .expect("Given an empty attempt folder, the part write should always succeed");
            writer
                .push_groups(vec![row(FILING_B, "Assets")])
                .await
                .expect("Given an empty attempt folder, the part write should always succeed");
            writer
                .finish(Vec::new(), Vec::new())
                .await
                .expect("Given whole digest groups, the unit should always finish")
        };
        let result: Vec<u64> = unit.row.files.iter().map(|file| file.row_count).collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_split_digest_group_when_two_calls_hold_one_digest_group() {
        let (_directory, store) = store();
        let mut writer = writer(&store, spec(1, 1)).await;
        writer
            .push_groups(vec![row(FILING_A, "Assets")])
            .await
            .expect("Given an empty attempt folder, the push should always succeed");
        writer
            .push_groups(vec![row(FILING_A, "Liabilities")])
            .await
            .expect("Given an empty attempt folder, the push should always succeed");

        let expected_result = Err(UnitWriteError::SplitDigestGroup(DuplicateDigestGroup::new(
            b"\x01\x00\x00\x00\x140000320193-24-000006",
        )));

        let result = writer.finish(Vec::new(), Vec::new()).await;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hash_the_inputs_in_key_order_when_the_inputs_come_unsorted() {
        let inputs = [input("sec/b.zip", "b"), input("sec/a.zip", "a")];

        // The digest of the byte form of the list `sec/a.zip`, then `sec/b.zip`.
        let expected_result = "25a2c3dcedee6077f7ced1c0021e8e20aa93a2a96fc23c4ad4dad483fb9c1514";

        let digest = input_digest(&inputs);
        let result = digest.as_str();

        assert_eq!(result, expected_result);
    }
}
