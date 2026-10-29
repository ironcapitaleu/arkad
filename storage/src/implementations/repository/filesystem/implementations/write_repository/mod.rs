//! # Write Repository
//!
//! Implements [`WriteRepository`] for [`FilesystemRepository`].

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::error::WriteError;
use crate::implementations::repository::filesystem::error::{
    FailedFilesystemOperation, FailedFilesystemOperationReason, FilesystemOperation,
};
use crate::implementations::repository::filesystem::shared::document_key::DocumentKey;
use crate::implementations::repository::filesystem::traits::access::CanWrite;
use crate::implementations::repository::filesystem::traits::document::Document;
use crate::implementations::repository::filesystem::{FilesystemRepository, check_content};
use crate::shared::Sha256Digest;
use crate::traits::repository::WriteRepository;

/// Counter that gives every temporary file of this process a distinct name.
static TEMPORARY_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Number of names [`create_temporary_file`] tries before it reports a name clash.
///
/// A clash needs a second process with the same process ID, or a file left by a crashed process.
const MAX_TEMPORARY_FILE_ATTEMPTS: u32 = 16;

#[async_trait]
impl<A: CanWrite, D: Document> WriteRepository for FilesystemRepository<A, D> {
    type Record = D;

    /// Writes a document and its metadata under the root.
    ///
    /// Checks the record before it touches a file, in this order:
    /// 1. The key does not end in
    ///    [`METADATA_SUFFIX`](crate::implementations::repository::filesystem::METADATA_SUFFIX).
    /// 2. The metadata holds the document type and document type version of `D`, and serializes as
    ///    JSON.
    /// 3. The bytes have the size and the SHA-256 digest the metadata names.
    ///
    /// Then `persist` checks that the root exists. `persist` writes no file and returns `Ok(())` if
    /// all three conditions hold:
    ///
    /// 1. The stored metadata equals the new metadata outside the run fields `produced_at`,
    ///    `execution_id`, and `ingestion_mode`.
    /// 2. The stored run fields are valid.
    /// 3. The stored document matches the digest of the new bytes.
    ///
    /// The stored run fields then stay. In every other case, `persist` creates every missing
    /// directory between the root and the document, and the new bytes and metadata replace the
    /// old. This case covers a torn pair, whose metadata matches the new bytes but whose
    /// document does not.
    ///
    /// # Errors
    ///
    /// Returns a [`WriteError::FailedIntegrityCheck`] that names the mismatch if the bytes have
    /// another size or digest than the record's metadata names. The write touches no file.
    ///
    /// Returns a [`WriteError::Backend`] that holds the [`FailedFilesystemOperation`] message if:
    /// - the key ends in the metadata suffix, or the record's metadata holds another document type or
    ///   document type version than `D`. The write touches no file
    ///   ([`BackendError::FailedOperation`](crate::BackendError::FailedOperation)).
    /// - the root does not exist, is not a directory, or has a file in its path
    ///   ([`BackendError::UnreachableStorage`](crate::BackendError::UnreachableStorage)).
    /// - the filesystem denies access to a path the write touches
    ///   ([`BackendError::UnauthorizedAccess`](crate::BackendError::UnauthorizedAccess)).
    /// - the filesystem rejects the write for any other reason, such as a full disk or a file
    ///   where a directory must be
    ///   ([`BackendError::FailedOperation`](crate::BackendError::FailedOperation)).
    async fn persist(&self, record: Self::Record) -> Result<(), WriteError> {
        let metadata = self
            .prepare_metadata(&record)
            .map_err(|error| WriteError::Backend(error.into()))?;
        check_content(record.bytes(), record.metadata())
            .map_err(|reason| WriteError::failed_integrity_check(reason.to_string()))?;
        let key = record.key();
        let bytes = record.bytes();
        let digest = record.metadata().sha256();
        self.write_files(key, &metadata, bytes, digest)
            .await
            .map_err(|error| WriteError::Backend(error.into()))
    }
}

impl<A: CanWrite, D: Document> FilesystemRepository<A, D> {
    /// Checks the record's key, checks that its metadata holds the document type and document type
    /// version of `D`, then serializes the metadata.
    ///
    /// Runs before the first `await`, so the write's future holds no reference to `D`, and `D`
    /// needs no `Sync` bound.
    fn prepare_metadata(&self, record: &D) -> Result<Vec<u8>, FailedFilesystemOperation> {
        self.check_key(record.key())?;
        let metadata_path = self.metadata_path(record.key());
        let metadata = record.metadata();
        let expected_document_type = D::DOCUMENT_TYPE.to_string();
        if metadata.document_type() != expected_document_type
            || metadata.document_type_version() != D::DOCUMENT_TYPE_VERSION
        {
            let reason = FailedFilesystemOperationReason::UnexpectedDocumentType {
                found_document_type: metadata.document_type().to_owned(),
                found_document_type_version: metadata.document_type_version(),
                expected_document_type,
                expected_document_type_version: D::DOCUMENT_TYPE_VERSION,
            };
            return Err(FailedFilesystemOperation::new(reason, metadata_path));
        }
        serde_json::to_vec_pretty(metadata).map_err(|_| {
            FailedFilesystemOperation::new(
                FailedFilesystemOperationReason::UnserializableMetadata,
                metadata_path,
            )
        })
    }
}

// `A: Access` makes `A` `Sync`, so the future of `write_files` is `Send`. The method takes no `D`,
// so `D` needs no bound.
impl<A: CanWrite, D> FilesystemRepository<A, D> {
    /// Checks the root. Unless the stored pair matches the new pair, creates the directories below
    /// the root, then writes the metadata and the document under the key.
    async fn write_files(
        &self,
        key: &DocumentKey,
        metadata: &[u8],
        bytes: &[u8],
        digest: &Sha256Digest,
    ) -> Result<(), FailedFilesystemOperation> {
        self.ensure_root_is_reachable().await?;

        let document_path = self.document_path(key);
        let metadata_path = self.metadata_path(key);
        if stored_pair_matches(&metadata_path, metadata, &document_path, digest).await {
            return Ok(());
        }

        if let Some(directory) = document_path.parent() {
            fs::create_dir_all(directory).await.map_err(|error| {
                FailedFilesystemOperation::from_io(
                    FilesystemOperation::CreateDirectory,
                    directory,
                    &error,
                )
            })?;
        }

        write_pair(&metadata_path, metadata, &document_path, bytes).await
    }
}

/// The metadata fields that name the run, which a skipped write keeps from the stored metadata.
#[derive(Deserialize)]
struct StoredRun {
    #[expect(
        dead_code,
        reason = "parsed only to check that the stored value is valid"
    )]
    produced_at: DateTime<Utc>,
    #[expect(
        dead_code,
        reason = "parsed only to check that the stored value is valid"
    )]
    execution_id: Uuid,
    #[expect(
        dead_code,
        reason = "parsed only to check that the stored value is valid"
    )]
    ingestion_mode: Option<String>,
}

/// Returns `true` if the stored metadata equals the new metadata outside the run fields, and the
/// stored document matches the digest.
///
/// Returns `false` if a file is missing or unreadable, a stored run field is invalid, or another
/// field differs, so the write repairs the pair.
async fn stored_pair_matches(
    metadata_path: &Path,
    metadata: &[u8],
    document_path: &Path,
    digest: &Sha256Digest,
) -> bool {
    let Ok(stored_json) = fs::read(metadata_path).await else {
        return false;
    };
    let (Ok(stored), Ok(new)) = (
        serde_json::from_slice::<Value>(&stored_json),
        serde_json::from_slice::<Value>(metadata),
    ) else {
        return false;
    };
    if StoredRun::deserialize(&stored).is_err() {
        return false;
    }
    if without_run_fields(stored) != without_run_fields(new) {
        return false;
    }
    fs::read(document_path)
        .await
        .is_ok_and(|document| Sha256Digest::of(document) == *digest)
}

/// Removes `produced_at`, `execution_id`, and `ingestion_mode` from a metadata object.
fn without_run_fields(mut metadata: Value) -> Value {
    if let Some(fields) = metadata.as_object_mut() {
        fields.remove("produced_at");
        fields.remove("execution_id");
        fields.remove("ingestion_mode");
    }
    metadata
}

/// Writes the metadata and the document to temporary files, then renames both into place.
///
/// Both temporary files are complete and synced before the first rename. If writing a temporary
/// file fails, no file under the key changes. The metadata is renamed first, so a document on disk
/// always has its metadata.
async fn write_pair(
    metadata_path: &Path,
    metadata: &[u8],
    document_path: &Path,
    document: &[u8],
) -> Result<(), FailedFilesystemOperation> {
    let metadata_temporary = write_temporary_file(metadata_path, metadata).await?;
    let document_temporary = match write_temporary_file(document_path, document).await {
        Ok(path) => path,
        Err(error) => {
            remove_temporary_file(&metadata_temporary).await;
            return Err(error);
        }
    };

    if let Err(error) = replace(&metadata_temporary, metadata_path).await {
        remove_temporary_file(&document_temporary).await;
        return Err(error);
    }
    replace(&document_temporary, document_path).await
}

/// Writes bytes to a new temporary file beside the target and syncs them to disk.
///
/// Returns the path of the temporary file. If a step fails, the function removes the file.
async fn write_temporary_file(
    target: &Path,
    bytes: &[u8],
) -> Result<PathBuf, FailedFilesystemOperation> {
    let (temporary, mut file) = create_temporary_file(target).await?;
    let result = write_and_sync(&mut file, &temporary, bytes).await;
    drop(file);

    match result {
        Ok(()) => Ok(temporary),
        Err(error) => {
            remove_temporary_file(&temporary).await;
            Err(error)
        }
    }
}

/// Creates a new temporary file beside the target and opens it for writing.
///
/// Opens the file only if no file exists at its path, so two writers never share one temporary
/// file. If the path exists, the function tries the next name.
async fn create_temporary_file(
    target: &Path,
) -> Result<(PathBuf, File), FailedFilesystemOperation> {
    let mut attempt = 1;
    loop {
        let temporary = temporary_path(target);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .await
        {
            Ok(file) => return Ok((temporary, file)),
            Err(error)
                if error.kind() == io::ErrorKind::AlreadyExists
                    && attempt < MAX_TEMPORARY_FILE_ATTEMPTS =>
            {
                attempt += 1;
            }
            Err(error) => {
                return Err(FailedFilesystemOperation::from_io(
                    FilesystemOperation::CreateFile,
                    &temporary,
                    &error,
                ));
            }
        }
    }
}

/// Writes all bytes to the file and syncs them to disk.
///
/// The flush comes before the sync because tokio reports an error from its last buffered write
/// only to a flush, not to a sync.
async fn write_and_sync(
    file: &mut File,
    path: &Path,
    bytes: &[u8],
) -> Result<(), FailedFilesystemOperation> {
    let write_error =
        |error| FailedFilesystemOperation::from_io(FilesystemOperation::Write, path, &error);
    file.write_all(bytes).await.map_err(write_error)?;
    file.flush().await.map_err(write_error)?;
    file.sync_all().await.map_err(|error| {
        FailedFilesystemOperation::from_io(FilesystemOperation::Sync, path, &error)
    })
}

/// Renames a temporary file over the target in one step.
///
/// The temporary file sits in the target's directory, so the rename stays on one filesystem. If
/// the rename fails, the function removes the temporary file.
async fn replace(temporary: &Path, target: &Path) -> Result<(), FailedFilesystemOperation> {
    let result = fs::rename(temporary, target).await.map_err(|error| {
        FailedFilesystemOperation::from_io(FilesystemOperation::Replace, target, &error)
    });
    if result.is_err() {
        remove_temporary_file(temporary).await;
    }
    result
}

/// Removes a temporary file after a failed write.
async fn remove_temporary_file(temporary: &Path) {
    // Deviation: the removal result is ignored. The write already failed, and that failure is the
    // error to report.
    let _ = fs::remove_file(temporary).await;
}

/// Returns a hidden path beside the target that no other write of this process uses.
fn temporary_path(target: &Path) -> PathBuf {
    let counter = TEMPORARY_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);

    let mut file_name = OsString::from(".");
    file_name.push(target.file_name().unwrap_or_default());
    file_name.push(format!(".{}.{counter}.tmp", process::id()));

    target.with_file_name(file_name)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};
    use tempfile::TempDir;

    use super::*;
    use crate::error::BackendError;
    use crate::implementations::repository::filesystem::{DocumentMetadata, ReadWrite, Write};
    use crate::tests::fixtures::sample_document::{
        SampleContext, SampleDocument, sample_document, sample_document_holding, sample_metadata,
    };

    /// Builds a repository with write access under the given root.
    fn repository_at(root: impl Into<PathBuf>) -> FilesystemRepository<Write, SampleDocument> {
        FilesystemRepository::builder()
            .writer()
            .document_type::<SampleDocument>()
            .root(root)
            .build()
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    /// Returns the names of the entries in a directory, sorted.
    fn sorted_file_names(directory: &Path) -> Vec<String> {
        let entries = std::fs::read_dir(directory)
            .expect("Given a directory this test created, listing it should always succeed");
        let mut names = Vec::new();
        for entry in entries {
            let entry =
                entry.expect("Given a readable directory, every entry should always be readable");
            let name = entry.file_name();
            names.push(
                name.into_string()
                    .expect("Given file names this test chose, each name should always be UTF-8"),
            );
        }
        names.sort();
        names
    }

    /// Reads and parses a metadata file the repository wrote.
    fn read_stored_metadata(metadata_path: &Path) -> DocumentMetadata<SampleContext> {
        let metadata_json = std::fs::read(metadata_path)
            .expect("Given a persisted record, its metadata file should always exist");
        serde_json::from_slice(&metadata_json)
            .expect("Given metadata the repository serialized, parsing it should always succeed")
    }

    const fn implements_write_repository<T: WriteRepository>() {}
    #[test]
    const fn should_implement_write_repository_for_filesystem_repository_with_write_access() {
        implements_write_repository::<FilesystemRepository<Write, SampleDocument>>();
    }

    #[test]
    const fn should_implement_write_repository_for_filesystem_repository_with_read_write_access() {
        implements_write_repository::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    #[tokio::test]
    async fn should_write_the_document_bytes_when_persisting_a_record() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let document_path = repository.document_path(sample_document.key());

        let expected_result = sample_document.bytes().to_vec();

        repository
            .persist(sample_document)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        let result = std::fs::read(document_path)
            .expect("Given a persisted record, its document file should always exist");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_the_metadata_beside_the_document_when_persisting_a_record() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let metadata_path = repository.metadata_path(sample_document.key());

        let expected_result = sample_document.metadata().clone();

        repository
            .persist(sample_document)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        let result = read_stored_metadata(&metadata_path);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_replace_the_document_when_persisting_a_record_under_an_existing_key() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let first_document = sample_document();
        let second_document = sample_document_holding(br#"{"cik":320193}"#, "2026-10-04T08:00:00Z");
        let document_path = repository.document_path(first_document.key());

        let expected_result = second_document.bytes().to_vec();

        repository
            .persist(first_document)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        repository
            .persist(second_document)
            .await
            .expect("Given an existing writable root, replacing a record should always succeed");
        let result = std::fs::read(document_path)
            .expect("Given a persisted record, its document file should always exist");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_replace_the_metadata_when_persisting_changed_bytes_under_an_existing_key() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let first_document = sample_document();
        let second_document = sample_document_holding(br#"{"cik":320193}"#, "2026-10-04T08:00:00Z");
        let metadata_path = repository.metadata_path(first_document.key());

        let expected_result = second_document.metadata().clone();

        repository
            .persist(first_document)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        repository
            .persist(second_document)
            .await
            .expect("Given an existing writable root, replacing a record should always succeed");
        let result = read_stored_metadata(&metadata_path);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_no_temporary_file_when_the_document_rename_fails() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let document_path = repository.document_path(sample_document.key());
        std::fs::create_dir_all(&document_path).expect(
            "Given a writable temp directory, creating subdirectories should always succeed",
        );
        std::fs::write(document_path.join("child"), b"")
            .expect("Given an existing directory, writing a file into it should always succeed");
        let directory = root.path().join("sec/companyfacts");

        let expected_result = vec![
            "CIK0000320193.json".to_owned(),
            "CIK0000320193.json.meta.json".to_owned(),
        ];

        // A file cannot replace a non-empty directory, so the document rename fails. The write
        // error is not the subject of this test. This test checks the disk afterwards.
        let _ = repository.persist(sample_document).await;
        let result = sorted_file_names(&directory);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_no_temporary_file_when_the_metadata_rename_fails() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let metadata_path = repository.metadata_path(sample_document.key());
        std::fs::create_dir_all(&metadata_path).expect(
            "Given a writable temp directory, creating subdirectories should always succeed",
        );
        std::fs::write(metadata_path.join("child"), b"")
            .expect("Given an existing directory, writing a file into it should always succeed");
        let directory = root.path().join("sec/companyfacts");

        let expected_result = vec!["CIK0000320193.json.meta.json".to_owned()];

        // A file cannot replace a non-empty directory, so the metadata rename fails. The write
        // error is not the subject of this test. This test checks the disk afterwards.
        let _ = repository.persist(sample_document).await;
        let result = sorted_file_names(&directory);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_only_the_document_and_its_metadata_when_persisting_a_record() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let directory = root.path().join("sec/companyfacts");

        let expected_result = vec![
            "CIK0000320193.json".to_owned(),
            "CIK0000320193.json.meta.json".to_owned(),
        ];

        repository
            .persist(sample_document)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        let result = sorted_file_names(&directory);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_root_when_persisting_under_a_root_that_does_not_exist() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let repository = repository_at(&root);
        let sample_document = sample_document();
        let failed_filesystem_operation =
            FailedFilesystemOperation::new(FailedFilesystemOperationReason::MissingRoot, &root);

        let expected_result = Err(WriteError::Backend(BackendError::from(
            failed_filesystem_operation,
        )));

        let result = repository.persist(sample_document).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_create_the_root_when_persisting_under_a_root_that_does_not_exist() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let repository = repository_at(&root);
        let sample_document = sample_document();

        let expected_result = false;

        // The write error is the subject of another test. This test checks the disk afterwards.
        let _ = repository.persist(sample_document).await;
        let result = root.exists();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_create_directory_when_a_file_stands_where_a_directory_must_be() {
        let root = temporary_root();
        std::fs::write(root.path().join("sec"), b"").expect(
            "Given a writable temp directory, writing a file into it should always succeed",
        );
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let directory = root.path().join("sec/companyfacts");
        let reason = FailedFilesystemOperationReason::FailedIo {
            operation: FilesystemOperation::CreateDirectory,
            kind: io::ErrorKind::NotADirectory,
        };
        let failed_filesystem_operation = FailedFilesystemOperation::new(reason, &directory);

        let expected_result = Err(WriteError::Backend(BackendError::from(
            failed_filesystem_operation,
        )));

        let result = repository.persist(sample_document).await;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_place_the_temporary_file_beside_the_target_for_a_target_path() {
        let target = Path::new("/data/arkad/sec/CIK0000320193.json");

        let expected_result = Some(Path::new("/data/arkad/sec"));

        let path = temporary_path(target);
        let result = path.parent();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_distinct_paths_for_two_calls_with_one_target() {
        let target = Path::new("/data/arkad/sec/CIK0000320193.json");

        let first_path = temporary_path(target);

        let result = temporary_path(target);

        assert_ne!(result, first_path);
    }

    /// Returns the sample document with the given field of its metadata set to `value`.
    ///
    /// Deserializing keeps any document type and document type version, so the result can hold a
    /// foreign document type.
    fn sample_document_with_metadata_field(
        field: &str,
        value: serde_json::Value,
    ) -> SampleDocument {
        let document = sample_document();
        let mut stored = serde_json::to_value(document.metadata())
            .expect("Given metadata of owned fields, serialization should always succeed");
        stored[field] = value;
        let metadata = serde_json::from_value(stored)
            .expect("Given JSON with every field, deserialization should always succeed");
        SampleDocument::from_parts(document.key().clone(), document.bytes().to_vec(), metadata)
    }

    #[tokio::test]
    async fn should_return_unexpected_document_type_when_persisting_a_record_of_another_document_type()
     {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let record = sample_document_with_metadata_field("document_type", "other_document".into());
        let metadata_path = repository.metadata_path(record.key());

        let expected_result = Err(WriteError::Backend(BackendError::from(
            FailedFilesystemOperation::new(
                FailedFilesystemOperationReason::UnexpectedDocumentType {
                    found_document_type: "other_document".to_owned(),
                    found_document_type_version: 1,
                    expected_document_type: "sample_document".to_owned(),
                    expected_document_type_version: 1,
                },
                metadata_path,
            ),
        )));

        let result = repository.persist(record).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unexpected_document_type_when_persisting_a_record_of_another_document_type_version()
     {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let record = sample_document_with_metadata_field("document_type_version", 2.into());
        let metadata_path = repository.metadata_path(record.key());

        let expected_result = Err(WriteError::Backend(BackendError::from(
            FailedFilesystemOperation::new(
                FailedFilesystemOperationReason::UnexpectedDocumentType {
                    found_document_type: "sample_document".to_owned(),
                    found_document_type_version: 2,
                    expected_document_type: "sample_document".to_owned(),
                    expected_document_type_version: 1,
                },
                metadata_path,
            ),
        )));

        let result = repository.persist(record).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_no_file_when_persisting_a_record_of_another_document_type() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let record = sample_document_with_metadata_field("document_type", "other_document".into());

        let expected_result: Vec<String> = Vec::new();

        // The write error is the subject of the two tests above. This test checks the disk
        // afterwards.
        let _ = repository.persist(record).await;
        let result = sorted_file_names(root.path());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_no_file_when_persisting_a_record_of_another_document_type_version() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let record = sample_document_with_metadata_field("document_type_version", 2.into());

        let expected_result: Vec<String> = Vec::new();

        // The write error is the subject of another test. This test checks the disk afterwards.
        let _ = repository.persist(record).await;
        let result = sorted_file_names(root.path());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_stored_metadata_when_persisting_unchanged_bytes() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let first_document = sample_document();
        let refetched_document = sample_document_holding(b"{}", "2026-10-04T08:00:00Z");
        let metadata_path = repository.metadata_path(first_document.key());

        let expected_result = first_document.metadata().clone();

        repository
            .persist(first_document)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        repository.persist(refetched_document).await.expect(
            "Given an existing writable root, persisting unchanged bytes should always succeed",
        );
        let result = read_stored_metadata(&metadata_path);

        assert_eq!(result, expected_result);
    }

    /// Returns the sample document with the given ingestion mode.
    fn sample_document_with_ingestion_mode(ingestion_mode: &str) -> SampleDocument {
        sample_document_with_metadata_field("ingestion_mode", ingestion_mode.into())
    }

    /// Returns the inode numbers of the metadata file and the document file.
    ///
    /// A write renames a new file into place, so a rewritten file gets a new inode.
    #[cfg(unix)]
    fn inodes(metadata_path: &Path, document_path: &Path) -> (u64, u64) {
        use std::os::unix::fs::MetadataExt;

        let inode = |path: &Path| {
            std::fs::metadata(path)
                .expect("Given a persisted record, its files should always exist")
                .ino()
        };
        (inode(metadata_path), inode(document_path))
    }

    #[tokio::test]
    async fn should_keep_the_stored_ingestion_mode_when_persisting_unchanged_bytes_with_another_ingestion_mode()
     {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let backfill_document = sample_document_with_ingestion_mode("backfill");
        let metadata_path = repository.metadata_path(backfill_document.key());

        let expected_result = Some("backfill".to_owned());

        repository
            .persist(backfill_document)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        repository
            .persist(sample_document_with_ingestion_mode("incremental"))
            .await
            .expect(
                "Given an existing writable root, persisting unchanged bytes should always succeed",
            );
        let result = read_stored_metadata(&metadata_path)
            .ingestion_mode()
            .map(str::to_owned);

        assert_eq!(result, expected_result);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn should_write_no_file_when_persisting_unchanged_bytes_with_another_ingestion_mode() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let backfill_document = sample_document_with_ingestion_mode("backfill");
        let metadata_path = repository.metadata_path(backfill_document.key());
        let document_path = repository.document_path(backfill_document.key());
        repository
            .persist(backfill_document)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");

        let expected_result = inodes(&metadata_path, &document_path);

        repository
            .persist(sample_document_with_ingestion_mode("incremental"))
            .await
            .expect(
                "Given an existing writable root, persisting unchanged bytes should always succeed",
            );
        let result = inodes(&metadata_path, &document_path);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_replace_the_metadata_when_the_stored_ingestion_mode_is_invalid() {
        let expected_result = sample_metadata();

        let result = persist_over_stored_metadata_field("ingestion_mode", 7.into()).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_replace_the_document_when_a_refetch_finds_a_torn_pair() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let old_document = sample_document();
        let new_document = sample_document_holding(br#"{"cik":320193}"#, "2026-10-04T08:00:00Z");
        let document_path = repository.document_path(old_document.key());
        let metadata_path = repository.metadata_path(old_document.key());
        let new_metadata_json = serde_json::to_vec(new_document.metadata())
            .expect("Given metadata of owned fields, serialization should always succeed");

        let expected_result = new_document.bytes().to_vec();

        repository
            .persist(old_document)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        // The new metadata beside the old document is the state a failed document rename leaves.
        std::fs::write(&metadata_path, new_metadata_json)
            .expect("Given an existing directory, overwriting the metadata should always succeed");
        repository
            .persist(new_document)
            .await
            .expect("Given an existing writable root, repairing a torn pair should always succeed");
        let result = std::fs::read(document_path)
            .expect("Given a persisted record, its document file should always exist");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_integrity_check_when_the_bytes_are_longer_than_the_metadata_names()
     {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let key = sample_document().key().clone();
        let record = SampleDocument::from_parts(key, b"{ }".to_vec(), sample_metadata());

        let expected_result = Err(WriteError::failed_integrity_check(
            "Document holds 3 bytes, metadata names 2 bytes",
        ));

        let result = repository.persist(record).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_integrity_check_when_the_bytes_differ_from_the_metadata_digest() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let key = sample_document().key().clone();
        let record = SampleDocument::from_parts(key, b"[]".to_vec(), sample_metadata());
        let reason = FailedFilesystemOperationReason::UnexpectedDocumentDigest {
            found_digest: Sha256Digest::of(b"[]"),
            expected_digest: sample_metadata().sha256().clone(),
        };

        let expected_result = Err(WriteError::failed_integrity_check(reason.to_string()));

        let result = repository.persist(record).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_stored_document_when_persisting_a_record_that_fails_the_integrity_check()
     {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let stored_document = sample_document();
        let key = stored_document.key().clone();
        let document_path = repository.document_path(&key);
        let record = SampleDocument::from_parts(key, b"[]".to_vec(), sample_metadata());

        let expected_result = stored_document.bytes().to_vec();

        repository
            .persist(stored_document)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        // The write error is the subject of the two tests above. This test checks the disk
        // afterwards.
        let _ = repository.persist(record).await;
        let result = std::fs::read(document_path)
            .expect("Given a persisted record, its document file should always exist");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_key_suffix_when_persisting_a_key_that_ends_in_the_metadata_suffix()
     {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json.meta.json")
            .expect("Given a valid relative path, the key should always build");
        let document_path = repository.document_path(&key);
        let record = SampleDocument::from_parts(key, b"{}".to_vec(), sample_metadata());

        let expected_result = Err(WriteError::Backend(BackendError::from(
            FailedFilesystemOperation::new(
                FailedFilesystemOperationReason::InvalidKeySuffix,
                document_path,
            ),
        )));

        let result = repository.persist(record).await;

        assert_eq!(result, expected_result);
    }

    /// Persists the sample document, sets one field of its stored metadata, persists the document
    /// again, and returns the stored metadata.
    async fn persist_over_stored_metadata_field(
        field: &str,
        value: serde_json::Value,
    ) -> DocumentMetadata<SampleContext> {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let record = sample_document();
        let metadata_path = repository.metadata_path(record.key());
        let mut stored = serde_json::to_value(record.metadata())
            .expect("Given metadata of owned fields, serialization should always succeed");
        stored[field] = value;
        let stored_json = serde_json::to_vec(&stored)
            .expect("Given a JSON value, serialization should always succeed");

        repository
            .persist(record.clone())
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        std::fs::write(&metadata_path, stored_json)
            .expect("Given an existing directory, overwriting the metadata should always succeed");
        repository
            .persist(record)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        read_stored_metadata(&metadata_path)
    }

    #[tokio::test]
    async fn should_replace_the_metadata_when_the_stored_metadata_holds_another_document_type_version()
     {
        let expected_result = sample_metadata();

        let result = persist_over_stored_metadata_field("document_type_version", 0.into()).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_replace_the_metadata_when_the_stored_metadata_holds_version_1() {
        let expected_result = sample_metadata();

        let result = persist_over_stored_metadata_field("metadata_version", 1.into()).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_replace_the_metadata_when_the_stored_produced_at_is_invalid() {
        let expected_result = sample_metadata();

        let result =
            persist_over_stored_metadata_field("produced_at", "not a timestamp".into()).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_replace_the_metadata_when_the_stored_execution_id_is_invalid() {
        let expected_result = sample_metadata();

        let result = persist_over_stored_metadata_field("execution_id", "not a uuid".into()).await;

        assert_eq!(result, expected_result);
    }
}
