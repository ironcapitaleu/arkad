//! # Part Writer
//!
//! Provides [`write_part()`], which encodes the rows of one part of a unit as Parquet and writes it
//! once, and [`PartLocation`], the unit and the task attempt that the path of the part names.
//!
//! ## Modules
//!
//! - [`constants`]: The row limit of a row group, the zstd level, and the footer keys.

use std::fmt;
use std::ops::Range;

use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression, ZstdLevel};
use parquet::errors::ParquetError;
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;
use storage::WriteRepository;
use storage::implementations::repository::filesystem::DocumentKey;
use storage::implementations::repository::write_once::Blob;
use storage::shared::{Crc32c, Sha256Digest};
use uuid::Uuid;

use self::constants::{
    FOOTER_DATASET, FOOTER_SCHEMA_VERSION, FOOTER_TRANSFORM_VERSION, FOOTER_UNIT_KEY,
    ROW_GROUP_ROWS, ZSTD_LEVEL,
};
use crate::datasets::implementations::UnitFile;
use crate::datasets::{ArrowRow, ArrowRowError, PartOrder, PartWriteError, SourceKind};
use crate::shared::row_digest::ByteForm;

pub mod constants;

/// The unit and the task attempt that the path of a part names.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PartLocation {
    /// The source of the unit. `None` for the series datasets, which have no `source=` folder.
    pub source: Option<SourceKind>,
    /// The key of the unit, for example `2024q1`. One component of a key.
    pub unit_key: String,
    /// The SHA-256 over the sorted list of input keys and digests. The path holds its first 16
    /// hex characters.
    pub input_digest: Sha256Digest,
    /// The version of the transform output.
    pub transform_version: u32,
    /// The job that writes the unit.
    pub execution_id: Uuid,
    /// The attempt of the task that writes the unit.
    pub attempt: TaskAttempt,
}

/// The attempt of a task: its number, and whether the split of its Notes file switched to buckets.
///
/// Prints as the number, plus `-b` after a switch, for example `1-b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskAttempt {
    /// The attempt number of the task, from 0.
    pub number: u32,
    /// Whether the split switched to buckets.
    pub is_bucket_split: bool,
}

impl fmt::Display for TaskAttempt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.number)?;
        if self.is_bucket_split {
            f.write_str("-b")?;
        }
        Ok(())
    }
}

/// Sorts the rows, encodes them as one Parquet part, and writes the part once.
///
/// The part goes to
/// `<dataset>/source=<source>/unit_key=<key>/<digest16>-v<version>/<execution_id>-<attempt>/part-<n>.parquet`.
/// The rows sort by their [`PartOrder`] row group key, then by their sort key. A row group holds at most
/// [`ROW_GROUP_ROWS`] rows, and starts anew where the row group key changes. Each column chunk
/// uses zstd at level [`ZSTD_LEVEL`]. The footer holds `dataset`, `schema_version`, `unit_key`,
/// and `transform_version`. The rows take the newest schema version of `R`.
///
/// # Returns
///
/// The [`UnitFile`] of the part, with the size and the CRC32C of the bytes the store received.
///
/// # Errors
///
/// Returns:
/// - [`PartWriteError::InvalidUnitKey`] if the unit key is not one component of a key.
/// - [`PartWriteError::FailedBatchConversion`] if `R` has no schema of its newest version, or the
///   rows do not turn into a record batch.
/// - [`PartWriteError::FailedPartEncoding`] if Parquet rejects the batches.
/// - [`PartWriteError::FailedPartWrite`] if the store does not write the part, for example
///   because a blob exists at its key.
///
/// # Examples
///
/// ```rust
/// use sec::datasets::{FilingRow, PartLocation, SourceKind, TaskAttempt, write_part};
/// use storage::implementations::repository::write_once::WriteOnceFilesystemRepository;
/// use storage::shared::Sha256Digest;
/// use uuid::Uuid;
///
/// let directory = tempfile::tempdir()
///     .expect("Given a writable temporary folder, the creation should always succeed");
/// let store = WriteOnceFilesystemRepository::new(directory.path());
/// let location = PartLocation {
///     source: Some(SourceKind::Notes),
///     unit_key: "2024q1".to_owned(),
///     input_digest: Sha256Digest::of(b""),
///     transform_version: 1,
///     execution_id: Uuid::nil(),
///     attempt: TaskAttempt { number: 0, is_bucket_split: false },
/// };
/// let rows: Vec<FilingRow> = Vec::new();
///
/// let expected_result = "filings/source=notes/unit_key=2024q1/e3b0c44298fc1c14-v1/00000000-0000-0000-0000-000000000000-0/part-0.parquet";
///
/// let runtime = tokio::runtime::Runtime::new()
///     .expect("Given a fresh process, the creation of a runtime should always succeed");
/// let file = runtime
///     .block_on(write_part(&store, &location, 0, rows))
///     .expect("Given an empty store and a valid unit key, the write should always succeed");
/// let result = file.path;
///
/// assert_eq!(result, expected_result);
/// ```
pub async fn write_part<R, S>(
    store: &S,
    location: &PartLocation,
    part_number: u32,
    mut rows: Vec<R>,
) -> Result<UnitFile, PartWriteError>
where
    R: ArrowRow + PartOrder,
    S: WriteRepository<Record = Blob>,
{
    let (path, key) = part_key(location, R::DATASET, part_number)?;
    let row_count = rows.len() as u64;
    let bytes = encode_part(location, &mut rows, ROW_GROUP_ROWS)?;
    drop(rows);
    let file = UnitFile {
        path,
        bytes: bytes.len() as u64,
        crc32c: Crc32c::of(&bytes),
        row_count,
    };
    store
        .persist(Blob::new(key, bytes))
        .await
        .map_err(|source| PartWriteError::FailedPartWrite { source })?;
    Ok(file)
}

/// Returns the path text of the part and its key.
fn part_key(
    location: &PartLocation,
    dataset: &str,
    part_number: u32,
) -> Result<(String, DocumentKey), PartWriteError> {
    let invalid_unit_key = || PartWriteError::InvalidUnitKey {
        unit_key: location.unit_key.clone(),
    };
    if location.unit_key.is_empty() || location.unit_key.contains('/') {
        return Err(invalid_unit_key());
    }
    let source = location
        .source
        .map(|source| format!("source={source}/"))
        .unwrap_or_default();
    let digest = &location.input_digest.as_str()[..16];
    let path = format!(
        "{dataset}/{source}unit_key={}/{digest}-v{}/{}-{}/part-{part_number}.parquet",
        location.unit_key, location.transform_version, location.execution_id, location.attempt,
    );
    // The unit key is the only component that the caller chooses, so a rejected key names it.
    let key = DocumentKey::new(&path).map_err(|_| invalid_unit_key())?;
    Ok((path, key))
}

/// Sorts the rows and encodes them as Parquet, with at most `row_group_rows` rows per row group.
fn encode_part<R: ArrowRow + PartOrder>(
    location: &PartLocation,
    rows: &mut [R],
    row_group_rows: usize,
) -> Result<Vec<u8>, PartWriteError> {
    rows.sort_by_cached_key(|row| {
        key_bytes(|byte_form| {
            row.write_row_group_key(byte_form);
            row.write_sort_key(byte_form);
        })
    });
    let schema = R::schema(R::SCHEMA_VERSION).ok_or(ArrowRowError::UnknownSchemaVersion {
        dataset: R::DATASET,
        version: R::SCHEMA_VERSION,
    })?;
    let properties = part_properties::<R>(location, row_group_rows)?;
    let mut writer = ArrowWriter::try_new(Vec::new(), schema, Some(properties))
        .map_err(|error| failed_encoding(&error))?;
    for range in row_group_ranges(rows, row_group_rows) {
        writer
            .write(&R::to_batch(&rows[range])?)
            .map_err(|error| failed_encoding(&error))?;
        writer.flush().map_err(|error| failed_encoding(&error))?;
    }
    writer.into_inner().map_err(|error| failed_encoding(&error))
}

/// Returns the writer properties of a part: zstd, the row limit, and the footer entries.
fn part_properties<R: ArrowRow>(
    location: &PartLocation,
    row_group_rows: usize,
) -> Result<WriterProperties, PartWriteError> {
    parquet_properties(footer::<R>(location), row_group_rows)
        .map_err(|error| failed_encoding(&error))
}

/// Returns the writer properties of a Parquet file of a dataset: zstd at level [`ZSTD_LEVEL`], the
/// row limit of a row group, and the footer entries.
pub(crate) fn parquet_properties(
    footer: Vec<KeyValue>,
    row_group_rows: usize,
) -> Result<WriterProperties, ParquetError> {
    let level = ZstdLevel::try_new(ZSTD_LEVEL)?;
    Ok(WriterProperties::builder()
        .set_compression(Compression::ZSTD(level))
        .set_max_row_group_row_count(Some(row_group_rows))
        .set_key_value_metadata(Some(footer))
        .build())
}

/// Returns the ranges of the row groups of sorted rows.
///
/// A range ends after `row_group_rows` rows, or before a row whose row group key differs from the
/// key of the first row of the range.
fn row_group_ranges<R: PartOrder>(rows: &[R], row_group_rows: usize) -> Vec<Range<usize>> {
    let group_key = |row: &R| key_bytes(|byte_form| row.write_row_group_key(byte_form));
    let mut ranges = Vec::new();
    let mut start = 0;
    let mut start_key = rows.first().map(group_key);
    for (index, row) in rows.iter().enumerate().skip(1) {
        let key = group_key(row);
        if index - start == row_group_rows || start_key.as_ref() != Some(&key) {
            ranges.push(start..index);
            start = index;
            start_key = Some(key);
        }
    }
    if start < rows.len() {
        ranges.push(start..rows.len());
    }
    ranges
}

/// Returns the footer entries of a part.
fn footer<R: ArrowRow>(location: &PartLocation) -> Vec<KeyValue> {
    vec![
        KeyValue::new(FOOTER_DATASET.to_owned(), R::DATASET.to_owned()),
        KeyValue::new(
            FOOTER_SCHEMA_VERSION.to_owned(),
            R::SCHEMA_VERSION.to_string(),
        ),
        KeyValue::new(FOOTER_UNIT_KEY.to_owned(), location.unit_key.clone()),
        KeyValue::new(
            FOOTER_TRANSFORM_VERSION.to_owned(),
            location.transform_version.to_string(),
        ),
    ]
}

/// Returns the bytes that `write` appends to an empty [`ByteForm`].
fn key_bytes(write: impl FnOnce(&mut ByteForm)) -> Vec<u8> {
    let mut byte_form = ByteForm::new();
    write(&mut byte_form);
    byte_form.into_bytes()
}

/// Wraps the text of a Parquet error in [`PartWriteError::FailedPartEncoding`].
fn failed_encoding(error: &ParquetError) -> PartWriteError {
    PartWriteError::FailedPartEncoding {
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow::array::{RecordBatch, StringArray, UInt8Array};
    use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
    use bytes::Bytes;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use parquet::file::metadata::RowGroupMetaData;
    use parquet::file::reader::{FileReader, SerializedFileReader};
    use parquet::schema::types::ColumnPath;
    use pretty_assertions::assert_eq;
    use storage::ReadRepository;
    use storage::WriteError;
    use storage::implementations::repository::write_once::WriteOnceFilesystemRepository;
    use tempfile::TempDir;

    use super::*;

    /// A row of a test dataset with a sort key and a row group key.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct FakeRow {
        filing_id: &'static str,
        bucket: u8,
    }

    impl ArrowRow for FakeRow {
        const DATASET: &'static str = "fake_rows";
        const SCHEMA_VERSION: u16 = 1;

        fn schema(version: u16) -> Option<SchemaRef> {
            (version == 1).then(|| {
                Arc::new(Schema::new(vec![
                    Field::new("filing_id", DataType::Utf8, false),
                    Field::new("bucket", DataType::UInt8, false),
                ]))
            })
        }

        fn to_batch(rows: &[Self]) -> Result<RecordBatch, ArrowRowError> {
            let filing_ids = StringArray::from_iter_values(rows.iter().map(|row| row.filing_id));
            let buckets = UInt8Array::from_iter_values(rows.iter().map(|row| row.bucket));
            let schema = Self::schema(1).expect("Given version 1, the schema should always exist");
            RecordBatch::try_new(schema, vec![Arc::new(filing_ids), Arc::new(buckets)]).map_err(
                |error| ArrowRowError::FailedBatchCreation {
                    dataset: Self::DATASET,
                    reason: error.to_string(),
                },
            )
        }

        fn from_batch(_batch: &RecordBatch, _version: u16) -> Result<Vec<Self>, ArrowRowError> {
            unimplemented!("The tests read the columns of a part directly")
        }
    }

    impl PartOrder for FakeRow {
        fn write_sort_key(&self, byte_form: &mut ByteForm) {
            byte_form.utf8(self.filing_id);
        }

        fn write_row_group_key(&self, byte_form: &mut ByteForm) {
            byte_form.integer(self.bucket);
        }
    }

    const fn row(filing_id: &'static str, bucket: u8) -> FakeRow {
        FakeRow { filing_id, bucket }
    }

    fn location(source: Option<SourceKind>, unit_key: &str) -> PartLocation {
        PartLocation {
            source,
            unit_key: unit_key.to_owned(),
            input_digest: Sha256Digest::of(b""),
            transform_version: 2,
            execution_id: Uuid::nil(),
            attempt: TaskAttempt {
                number: 1,
                is_bucket_split: false,
            },
        }
    }

    fn store() -> (TempDir, WriteOnceFilesystemRepository) {
        let directory = tempfile::tempdir()
            .expect("Given a writable temporary folder, the creation should always succeed");
        let store = WriteOnceFilesystemRepository::new(directory.path());
        (directory, store)
    }

    async fn stored_bytes(store: &WriteOnceFilesystemRepository, path: &str) -> Bytes {
        let key = DocumentKey::new(path)
            .expect("Given the path of a written part, the key should always build");
        store
            .get(key)
            .await
            .expect("Given a readable store, the read should always succeed")
            .expect("Given the path of a written part, the blob should always exist")
            .bytes
    }

    fn encoded(rows: &mut [FakeRow], row_group_rows: usize) -> Bytes {
        let location = location(Some(SourceKind::Notes), "2024q1");
        Bytes::from(
            encode_part(&location, rows, row_group_rows)
                .expect("Given rows of a valid schema, the encoding should always succeed"),
        )
    }

    fn file_reader(bytes: Bytes) -> SerializedFileReader<Bytes> {
        SerializedFileReader::new(bytes)
            .expect("Given the bytes of a written part, the footer should always parse")
    }

    #[test]
    fn should_print_the_number_when_the_split_did_not_switch() {
        let attempt = TaskAttempt {
            number: 0,
            is_bucket_split: false,
        };

        let expected_result = "0";

        let result = attempt.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_number_and_b_when_the_split_switched_to_buckets() {
        let attempt = TaskAttempt {
            number: 1,
            is_bucket_split: true,
        };

        let expected_result = "1-b";

        let result = attempt.to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_the_part_below_its_source_when_the_unit_has_a_source() {
        let (_directory, store) = store();
        let location = location(Some(SourceKind::Instance), "2026-11-06");
        let rows = vec![row("0000320193-24-000006", 0)];

        let expected_result = "fake_rows/source=instance/unit_key=2026-11-06/e3b0c44298fc1c14-v2/00000000-0000-0000-0000-000000000000-1/part-3.parquet";

        let file = write_part(&store, &location, 3, rows)
            .await
            .expect("Given an empty store and a valid unit key, the write should always succeed");
        let result = file.path;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_out_the_source_folder_when_the_unit_has_no_source() {
        let (_directory, store) = store();
        let location = location(None, "base-t0");
        let rows = vec![row("0000320193-24-000006", 0)];

        let expected_result = "fake_rows/unit_key=base-t0/e3b0c44298fc1c14-v2/00000000-0000-0000-0000-000000000000-1/part-0.parquet";

        let file = write_part(&store, &location, 0, rows)
            .await
            .expect("Given an empty store and a valid unit key, the write should always succeed");
        let result = file.path;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_unit_key_when_the_unit_key_holds_a_slash() {
        let (_directory, store) = store();
        let location = location(Some(SourceKind::Notes), "2024/q1");
        let rows = vec![row("0000320193-24-000006", 0)];

        let expected_result = Err(PartWriteError::InvalidUnitKey {
            unit_key: "2024/q1".to_owned(),
        });

        let result = write_part(&store, &location, 0, rows).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_unit_key_when_the_unit_key_is_empty() {
        let (_directory, store) = store();
        let location = location(None, "");
        let rows = vec![row("0000320193-24-000006", 0)];

        let expected_result = Err(PartWriteError::InvalidUnitKey {
            unit_key: String::new(),
        });

        let result = write_part(&store, &location, 0, rows).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_unit_key_when_the_unit_key_holds_a_colon() {
        let (_directory, store) = store();
        let location = location(None, "2026-11-06T06:00Z");
        let rows = vec![row("0000320193-24-000006", 0)];

        let expected_result = Err(PartWriteError::InvalidUnitKey {
            unit_key: "2026-11-06T06:00Z".to_owned(),
        });

        let result = write_part(&store, &location, 0, rows).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_part_write_when_a_blob_exists_at_the_key() {
        let (_directory, store) = store();
        let location = location(None, "base-t0");
        write_part(&store, &location, 0, vec![row("0000320193-24-000006", 0)])
            .await
            .expect(
                "Given an empty store and a valid unit key, the first write should always succeed",
            );
        let rows = vec![row("0000320193-24-000007", 0)];

        let expected_result = Err(PartWriteError::FailedPartWrite {
            source: WriteError::conflicting_write(
                "Blob 'fake_rows/unit_key=base-t0/e3b0c44298fc1c14-v2/00000000-0000-0000-0000-000000000000-1/part-0.parquet' exists",
            ),
        });

        let result = write_part(&store, &location, 0, rows).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_size_and_crc32c_of_the_stored_bytes_for_a_written_part() {
        let (_directory, store) = store();
        let location = location(Some(SourceKind::Notes), "2024q1");
        let rows = vec![row("0000320193-24-000006", 0)];
        let file = write_part(&store, &location, 0, rows)
            .await
            .expect("Given an empty store and a valid unit key, the write should always succeed");
        let bytes = stored_bytes(&store, &file.path).await;

        // The bytes depend on the Parquet encoder, so the test compares the file with the stored
        // bytes. The tests of `Crc32c` pin the checksum.
        let expected_result = (bytes.len() as u64, Crc32c::of(&bytes), 1);

        let result = (file.bytes, file.crc32c, file.row_count);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_sort_the_rows_by_their_sort_key_when_the_rows_come_unsorted() {
        let mut rows = vec![
            row("0000320193-24-000009", 0),
            row("0000320193-24-000006", 0),
            row("0000320193-24-000007", 0),
        ];

        let expected_result = vec![
            Some("0000320193-24-000006"),
            Some("0000320193-24-000007"),
            Some("0000320193-24-000009"),
        ];

        let batch = ParquetRecordBatchReaderBuilder::try_new(encoded(&mut rows, ROW_GROUP_ROWS))
            .expect("Given the bytes of a written part, the footer should always parse")
            .build()
            .expect("Given a parsed footer, the reader should always build")
            .next()
            .expect("Given a part with rows, a batch should always exist")
            .expect("Given a valid part, the batch should always decode");
        let filing_ids = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("Given the schema of the test rows, column 0 should always hold text");
        let result: Vec<Option<&str>> = filing_ids.iter().collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_footer_entries_for_a_written_part() {
        let mut rows = vec![row("0000320193-24-000006", 0)];

        let expected_result = vec![
            ("dataset".to_owned(), Some("fake_rows".to_owned())),
            ("schema_version".to_owned(), Some("1".to_owned())),
            ("unit_key".to_owned(), Some("2024q1".to_owned())),
            ("transform_version".to_owned(), Some("2".to_owned())),
        ];

        let reader = file_reader(encoded(&mut rows, ROW_GROUP_ROWS));
        let entries = reader
            .metadata()
            .file_metadata()
            .key_value_metadata()
            .expect("Given a part with footer entries, the entries should always exist");
        let result: Vec<(String, Option<String>)> = entries
            .iter()
            .filter(|entry| entry.key != "ARROW:schema")
            .map(|entry| (entry.key.clone(), entry.value.clone()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_compress_the_column_chunks_with_zstd_for_a_written_part() {
        let mut rows = vec![row("0000320193-24-000006", 0)];

        // The footer names the codec, not its level, so a reader gives the default level.
        let expected_result = Compression::ZSTD(ZstdLevel::default());

        let reader = file_reader(encoded(&mut rows, ROW_GROUP_ROWS));
        let result = reader.metadata().row_group(0).column(0).compression();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_one_row_group_per_range_when_the_rows_pass_the_row_limit() {
        let mut rows = vec![
            row("0000320193-24-000006", 0),
            row("0000320193-24-000007", 0),
            row("0000320193-24-000008", 0),
        ];

        let expected_result = vec![2, 1];

        let reader = file_reader(encoded(&mut rows, 2));
        let result: Vec<i64> = reader
            .metadata()
            .row_groups()
            .iter()
            .map(RowGroupMetaData::num_rows)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_set_zstd_at_level_3_for_each_column_of_a_part() {
        let location = location(None, "2024q1");

        let expected_result = Compression::ZSTD(
            ZstdLevel::try_new(3).expect("Given level 3, the zstd level should always build"),
        );

        let properties = part_properties::<FakeRow>(&location, ROW_GROUP_ROWS)
            .expect("Given the zstd level of the writer, the properties should always build");
        let result = properties.compression(&ColumnPath::from("filing_id"));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_rows_of_one_row_group_key_together_when_the_sort_key_mixes_them() {
        let mut rows = vec![
            row("0000320193-24-000006", 2),
            row("0000320193-24-000007", 1),
            row("0000320193-24-000008", 2),
        ];

        let expected_result = vec![1, 2];

        let reader = file_reader(encoded(&mut rows, ROW_GROUP_ROWS));
        let result: Vec<i64> = reader
            .metadata()
            .row_groups()
            .iter()
            .map(RowGroupMetaData::num_rows)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_start_a_row_group_where_the_row_group_key_changes() {
        let rows = [
            row("0000320193-24-000006", 1),
            row("0000320193-24-000007", 1),
            row("0000320193-24-000008", 2),
        ];

        let expected_result = vec![0..2, 2..3];

        let result = row_group_ranges(&rows, ROW_GROUP_ROWS);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_start_a_row_group_after_the_row_limit_when_the_key_does_not_change() {
        let rows = [
            row("0000320193-24-000006", 1),
            row("0000320193-24-000007", 1),
            row("0000320193-24-000008", 1),
        ];

        let expected_result = vec![0..2, 2..3];

        let result = row_group_ranges(&rows, 2);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_row_group_when_the_part_has_no_rows() {
        let rows: [FakeRow; 0] = [];

        let expected_result: Vec<Range<usize>> = Vec::new();

        let result = row_group_ranges(&rows, ROW_GROUP_ROWS);

        assert_eq!(result, expected_result);
    }
}
