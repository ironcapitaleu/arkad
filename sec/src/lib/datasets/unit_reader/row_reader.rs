//! # Row Reader
//!
//! Provides [`read_rows()`], which reads the rows of one unit from the parts that its
//! `unit_manifests` row names.

use bytes::Bytes;
use parquet::arrow::arrow_reader::{
    ArrowReaderMetadata, ArrowReaderOptions, ParquetRecordBatchReaderBuilder,
};
use storage::ReadRepository;
use storage::implementations::repository::filesystem::DocumentKey;
use storage::implementations::repository::write_once::Blob;
use storage::shared::Crc32c;

use super::RowFilter;
use crate::datasets::{ArrowRow, UnitManifestRow, UnitReadError};

/// Returns the rows of the unit that the filter keeps, as rows of type `R`.
///
/// Reads each part of `unit.files` whole through [`ReadRepository`], in the order of the list, and
/// checks its size and CRC32C against the entry of the list. It skips the row groups that the filter excludes by their statistics, and decodes the other row
/// groups with the `schema_version` of the unit. So a unit of an older schema version reads into
/// the current row type.
///
/// # Errors
///
/// Returns:
/// - [`UnitReadError::UnexpectedDataset`] if the unit belongs to another dataset than `R`.
/// - [`UnitReadError::FailedPartRead`] if the store does not read a part.
/// - [`UnitReadError::MissingPartFile`] if a part of the unit is not in the store.
/// - [`UnitReadError::InvalidPartChecksum`] if the size or the CRC32C of a part differs from the
///   entry of the unit.
/// - [`UnitReadError::FailedPartDecoding`] if a part is not Parquet, or lacks the column of the
///   filter.
/// - [`UnitReadError::FailedPartRowConversion`] if the batches of a part do not turn into rows.
///
/// # Examples
///
/// ```rust
/// use sec::datasets::{ArrowRow, FilingRow, RowFilter, SourceKind, current_units, read_rows};
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
/// let rows = runtime
///     .block_on(async {
///         let mut rows: Vec<FilingRow> = Vec::new();
///         for unit in current_units(&store, FilingRow::DATASET, Some(SourceKind::Notes))
///             .await?
///             .values()
///         {
///             rows.extend(read_rows(&store, unit, &RowFilter::All).await?);
///         }
///         Ok::<_, sec::datasets::UnitReadError>(rows)
///     })
///     .expect("Given an empty store, the read should always succeed");
/// let result = rows.len();
///
/// assert_eq!(result, expected_result);
/// ```
pub async fn read_rows<R, S>(
    store: &S,
    unit: &UnitManifestRow,
    filter: &RowFilter,
) -> Result<Vec<R>, UnitReadError>
where
    R: ArrowRow,
    S: ReadRepository<Key = DocumentKey, Record = Blob>,
{
    if unit.dataset != R::DATASET {
        return Err(UnitReadError::UnexpectedDataset {
            dataset: unit.dataset.clone(),
            expected: R::DATASET,
        });
    }
    let mut rows = Vec::new();
    for file in &unit.files {
        let missing = || UnitReadError::MissingPartFile {
            path: file.path.clone(),
        };
        // A path that no key can hold names no file of the store.
        let key = DocumentKey::new(&file.path).map_err(|_| missing())?;
        let blob = store
            .get(key)
            .await
            .map_err(|source| UnitReadError::FailedPartRead {
                path: file.path.clone(),
                source,
            })?
            .ok_or_else(missing)?;
        if blob.bytes.len() as u64 != file.bytes || Crc32c::of(&blob.bytes) != file.crc32c {
            return Err(UnitReadError::InvalidPartChecksum {
                path: file.path.clone(),
            });
        }
        rows.extend(decode_part(
            &file.path,
            blob.bytes,
            unit.schema_version,
            filter,
        )?);
    }
    Ok(rows)
}

/// Decodes the rows of one part that the filter keeps, with the schema version of its unit.
fn decode_part<R: ArrowRow>(
    path: &str,
    bytes: Bytes,
    version: u16,
    filter: &RowFilter,
) -> Result<Vec<R>, UnitReadError> {
    let failed_decoding = |reason: String| UnitReadError::FailedPartDecoding {
        path: path.to_owned(),
        reason,
    };
    let metadata = ArrowReaderMetadata::load(&bytes, ArrowReaderOptions::new())
        .map_err(|error| failed_decoding(error.to_string()))?;
    let row_groups = filter.row_groups(&metadata).map_err(failed_decoding)?;
    let mut builder = ParquetRecordBatchReaderBuilder::new_with_metadata(bytes, metadata);
    if let Some(row_groups) = row_groups {
        builder = builder.with_row_groups(row_groups);
    }
    let reader = builder
        .build()
        .map_err(|error| failed_decoding(error.to_string()))?;
    let mut rows = Vec::new();
    for batch in reader {
        let batch = batch.map_err(|error| failed_decoding(error.to_string()))?;
        let batch = filter.apply(batch).map_err(failed_decoding)?;
        rows.extend(R::from_batch(&batch, version).map_err(|source| {
            UnitReadError::FailedPartRowConversion {
                path: path.to_owned(),
                source: Box::new(source),
            }
        })?);
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use pretty_assertions::assert_eq;
    use storage::implementations::repository::write_once::WriteOnceFilesystemRepository;

    use super::*;
    use crate::datasets::implementations::UnitFile;
    use crate::datasets::unit_reader::fixtures::{
        FakeRow, FakeRowV1, notes_unit, part_without_buckets, store, unit_with_part, write_units,
    };
    use crate::datasets::{SourceKind, current_units};

    const FILING_A: &str = "0000320193-24-000006";
    const FILING_B: &str = "0000320193-24-000007";
    const FILING_C: &str = "0000320193-24-000008";

    fn fake_row(filing_id: &str, entity_bucket: u8, unit: Option<&str>) -> FakeRow {
        FakeRow {
            filing_id: filing_id.to_owned(),
            entity_bucket,
            unit: unit.map(str::to_owned),
        }
    }

    async fn read(
        store: &WriteOnceFilesystemRepository,
        unit: &UnitManifestRow,
        filter: &RowFilter,
    ) -> Result<Vec<FakeRow>, UnitReadError> {
        read_rows::<FakeRow, _>(store, unit, filter).await
    }

    #[tokio::test]
    async fn should_read_the_rows_of_both_versions_when_two_units_have_two_schema_versions() {
        let (_directory, store) = store();
        let old_rows = vec![FakeRowV1 {
            filing_id: FILING_A,
            entity_bucket: 3,
        }];
        let new_rows = vec![fake_row(FILING_B, 5, Some("USD"))];
        let old_unit = unit_with_part(&store, notes_unit("2024q1", 1), old_rows).await;
        let new_unit = unit_with_part(&store, notes_unit("2024q2", 1), new_rows).await;
        write_units(
            &store,
            0,
            0,
            vec![old_unit, new_unit],
            "2026-11-28T12:00:00Z",
        )
        .await;

        let expected_result = vec![
            fake_row(FILING_A, 3, None),
            fake_row(FILING_B, 5, Some("USD")),
        ];

        let units = current_units(&store, FakeRow::DATASET, Some(SourceKind::Notes))
            .await
            .expect("Given list files that the writer wrote, the read should always succeed");
        let mut result = Vec::new();
        for unit in units.values() {
            result.extend(
                read(&store, unit, &RowFilter::All)
                    .await
                    .expect("Given parts that the writer wrote, the read should always succeed"),
            );
        }

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_only_the_rows_of_the_buckets_when_the_filter_names_buckets() {
        let (_directory, store) = store();
        let rows = vec![
            fake_row(FILING_A, 3, None),
            fake_row(FILING_B, 5, None),
            fake_row(FILING_C, 7, None),
        ];
        let unit = unit_with_part(&store, notes_unit("2024q1", 1), rows).await;
        let filter = RowFilter::EntityBuckets(BTreeSet::from([5, 7]));

        let expected_result = Ok(vec![
            fake_row(FILING_B, 5, None),
            fake_row(FILING_C, 7, None),
        ]);

        let result = read(&store, &unit, &filter).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_part_file_when_a_part_of_the_unit_is_gone() {
        let (_directory, store) = store();
        let path = "fake_rows/source=notes/unit_key=2024q1/part-0.parquet";
        let unit = UnitManifestRow {
            files: vec![UnitFile {
                path: path.to_owned(),
                bytes: 0,
                crc32c: Crc32c::of(b""),
                row_count: 0,
            }],
            ..notes_unit("2024q1", 1)
        };

        let expected_result = Err(UnitReadError::MissingPartFile {
            path: path.to_owned(),
        });

        let result = read(&store, &unit, &RowFilter::All).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unexpected_dataset_when_the_unit_holds_another_dataset() {
        let (_directory, store) = store();
        let unit = UnitManifestRow {
            dataset: "filings".to_owned(),
            ..notes_unit("2024q1", 1)
        };

        let expected_result = Err(UnitReadError::UnexpectedDataset {
            dataset: "filings".to_owned(),
            expected: "fake_rows",
        });

        let result = read(&store, &unit, &RowFilter::All).await;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_part_decoding_when_the_part_lacks_the_bucket_column() {
        let bytes = part_without_buckets();
        let filter = RowFilter::EntityBuckets(BTreeSet::from([5]));

        let expected_result = Err(UnitReadError::FailedPartDecoding {
            path: "fake_rows/part-0.parquet".to_owned(),
            reason: "Arrow: Column 'entity_bucket' not found in schema for statistics conversion"
                .to_owned(),
        });

        let result = decode_part::<FakeRow>("fake_rows/part-0.parquet", bytes, 1, &filter);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_part_checksum_when_the_part_differs_from_its_entry() {
        let (_directory, store) = store();
        let rows = vec![fake_row(FILING_A, 3, None)];
        let unit = unit_with_part(&store, notes_unit("2024q1", 1), rows).await;
        let path = unit.files[0].path.clone();
        let mut files = unit.files.clone();
        files[0].crc32c = Crc32c::of(b"other bytes");
        let unit = UnitManifestRow { files, ..unit };

        let expected_result = Err(UnitReadError::InvalidPartChecksum { path });

        let result = read(&store, &unit, &RowFilter::All).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_part_checksum_when_the_size_differs_from_its_entry() {
        let (_directory, store) = store();
        let rows = vec![fake_row(FILING_A, 3, None)];
        let unit = unit_with_part(&store, notes_unit("2024q1", 1), rows).await;
        let path = unit.files[0].path.clone();
        let mut files = unit.files.clone();
        files[0].bytes += 1;
        let unit = UnitManifestRow { files, ..unit };

        let expected_result = Err(UnitReadError::InvalidPartChecksum { path });

        let result = read(&store, &unit, &RowFilter::All).await;

        assert_eq!(result, expected_result);
    }
}
