//! # Unit Reader
//!
//! Provides [`current_units()`], which reads the `unit_manifests` list files and returns the
//! current unit of each unit key of a dataset.
//!
//! The reader reads the list files only. It never lists a folder of a dataset, because a listing
//! also finds the parts of replaced units and of failed attempts.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use bytes::Bytes;
use futures_util::TryStreamExt;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use storage::implementations::repository::filesystem::{DocumentKey, KeyPrefix};
use storage::implementations::repository::write_once::Blob;
use storage::{ListRepository, ReadRepository};

use crate::datasets::implementations::UnitKind;
use crate::datasets::part_writer::constants::FOOTER_SCHEMA_VERSION;
use crate::datasets::{ArrowRow, SourceKind, UnitManifestRow, UnitReadError};

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
    let mut units = CurrentUnits::new();
    for row in manifest_rows(store).await? {
        if row.dataset != dataset || row.source != source {
            continue;
        }
        match units.entry((row.unit_kind, row.unit_key.clone())) {
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
    Ok(units)
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

    use super::*;

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
