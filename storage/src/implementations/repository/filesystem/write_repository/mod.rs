//! # Write Repository
//!
//! Implements [`WriteRepository`] for [`FilesystemRepository`].

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::AsyncWriteExt;

use super::raw_document::RawDocument;
use super::{FilesystemRepository, backend_error};
use crate::error::{BackendError, WriteError};
use crate::traits::repository::WriteRepository;

/// Counter that gives every temporary file of this process a distinct name.
static TEMPORARY_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Number of names [`create_temporary_file`] tries before it reports a name clash.
///
/// A clash needs a second process with the same process ID, or a file left by a crashed process.
const MAX_TEMPORARY_FILE_ATTEMPTS: u32 = 16;

#[async_trait]
impl WriteRepository for FilesystemRepository {
    type Record = RawDocument;

    /// Writes a document and its metadata under the root.
    ///
    /// Checks that the root exists, then creates every missing directory between the root and
    /// the document. If the key already names a document, the new bytes and metadata replace the
    /// old.
    ///
    /// # Errors
    ///
    /// Returns a [`WriteError::Backend`] if:
    /// - the root is missing, is not a directory, or has a file in its path
    ///   ([`BackendError::UnreachableStorage`]).
    /// - the filesystem denies access to a path the write touches
    ///   ([`BackendError::UnauthorizedAccess`]).
    /// - the filesystem rejects the write for any other reason, such as a full disk or a file
    ///   where a directory must be ([`BackendError::FailedOperation`]).
    async fn persist(&self, record: Self::Record) -> Result<(), WriteError> {
        self.ensure_root_is_reachable().await?;

        let document_path = self.document_path(record.key());
        if let Some(directory) = document_path.parent() {
            fs::create_dir_all(directory)
                .await
                .map_err(|error| backend_error("create directory", directory, &error))?;
        }

        let metadata = serde_json::to_vec_pretty(record.metadata()).map_err(|error| {
            BackendError::failed_operation(format!("Failed to serialize the metadata, {error}"))
        })?;
        let metadata_path = self.metadata_path(record.key());

        write_pair(&metadata_path, &metadata, &document_path, record.bytes()).await?;
        Ok(())
    }
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
) -> Result<(), BackendError> {
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
async fn write_temporary_file(target: &Path, bytes: &[u8]) -> Result<PathBuf, BackendError> {
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
async fn create_temporary_file(target: &Path) -> Result<(PathBuf, File), BackendError> {
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
            Err(error) => return Err(backend_error("create", &temporary, &error)),
        }
    }
}

/// Writes all bytes to the file and syncs them to disk.
///
/// The flush comes before the sync because tokio reports an error from its last buffered write
/// only to a flush, not to a sync.
async fn write_and_sync(file: &mut File, path: &Path, bytes: &[u8]) -> Result<(), BackendError> {
    file.write_all(bytes)
        .await
        .map_err(|error| backend_error("write", path, &error))?;
    file.flush()
        .await
        .map_err(|error| backend_error("write", path, &error))?;
    file.sync_all()
        .await
        .map_err(|error| backend_error("sync", path, &error))
}

/// Renames a temporary file over the target in one step.
///
/// The temporary file sits in the target's directory, so the rename stays on one filesystem. If
/// the rename fails, the function removes the temporary file.
async fn replace(temporary: &Path, target: &Path) -> Result<(), BackendError> {
    let result = fs::rename(temporary, target)
        .await
        .map_err(|error| backend_error("replace", target, &error));
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
    use chrono::{TimeZone, Utc};
    use pretty_assertions::{assert_eq, assert_ne};
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::repository::filesystem::DocumentMetadata;
    use crate::tests::fixtures::sample_raw_document::sample_raw_document;

    fn temporary_root() -> TempDir {
        TempDir::new()
            .expect("Given a writable system temp directory, creating a directory in it should always succeed")
    }

    const fn implements_write_repository<T: WriteRepository>() {}
    #[test]
    const fn should_implement_write_repository_for_filesystem_repository() {
        implements_write_repository::<FilesystemRepository>();
    }

    #[tokio::test]
    async fn should_write_the_document_bytes_when_persisting_a_record() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());
        let record = sample_raw_document();
        let document_path = repository.document_path(record.key());

        let expected_result = record.bytes().to_vec();

        repository
            .persist(record)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        let result = std::fs::read(document_path)
            .expect("Given a persisted record, its document file should always exist");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_the_metadata_beside_the_document_when_persisting_a_record() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());
        let record = sample_raw_document();
        let metadata_path = repository.metadata_path(record.key());

        let expected_result = record.metadata().clone();

        repository
            .persist(record)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        let metadata_json = std::fs::read(metadata_path)
            .expect("Given a persisted record, its metadata file should always exist");
        let result: DocumentMetadata = serde_json::from_slice(&metadata_json)
            .expect("Given metadata the repository serialized, parsing it should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_replace_the_document_when_persisting_a_record_under_an_existing_key() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());
        let first = sample_raw_document();
        let second = RawDocument::new(
            first.key().clone(),
            br#"{"cik":320193}"#.to_vec(),
            first.metadata().clone(),
        );
        let document_path = repository.document_path(first.key());

        let expected_result = second.bytes().to_vec();

        repository
            .persist(first)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        repository
            .persist(second)
            .await
            .expect("Given an existing writable root, replacing a record should always succeed");
        let result = std::fs::read(document_path)
            .expect("Given a persisted record, its document file should always exist");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_replace_the_metadata_when_persisting_a_record_under_an_existing_key() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());
        let first = sample_raw_document();
        let mut second_metadata = first.metadata().clone();
        second_metadata.fetched_at = Utc
            .with_ymd_and_hms(2026, 10, 4, 8, 0, 0)
            .single()
            .expect("Given a hardcoded valid timestamp, the conversion should always succeed");
        let second = RawDocument::new(first.key().clone(), first.bytes().to_vec(), second_metadata);
        let metadata_path = repository.metadata_path(first.key());

        let expected_result = second.metadata().clone();

        repository
            .persist(first)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        repository
            .persist(second)
            .await
            .expect("Given an existing writable root, replacing a record should always succeed");
        let metadata_json = std::fs::read(metadata_path)
            .expect("Given a persisted record, its metadata file should always exist");
        let result: DocumentMetadata = serde_json::from_slice(&metadata_json)
            .expect("Given metadata the repository serialized, parsing it should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_no_temporary_file_when_the_document_rename_fails() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());
        let record = sample_raw_document();
        let document_path = repository.document_path(record.key());
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
        let _ = repository.persist(record).await;
        let mut result: Vec<String> = std::fs::read_dir(directory)
            .expect("Given a directory this test created, listing it should always succeed")
            .map(|entry| {
                entry
                    .expect("Given a readable directory, every entry should always be readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        result.sort();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_no_temporary_file_when_the_metadata_rename_fails() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());
        let record = sample_raw_document();
        let metadata_path = repository.metadata_path(record.key());
        std::fs::create_dir_all(&metadata_path).expect(
            "Given a writable temp directory, creating subdirectories should always succeed",
        );
        std::fs::write(metadata_path.join("child"), b"")
            .expect("Given an existing directory, writing a file into it should always succeed");
        let directory = root.path().join("sec/companyfacts");

        let expected_result = vec!["CIK0000320193.json.meta.json".to_owned()];

        // A file cannot replace a non-empty directory, so the metadata rename fails. The write
        // error is not the subject of this test. This test checks the disk afterwards.
        let _ = repository.persist(record).await;
        let mut result: Vec<String> = std::fs::read_dir(directory)
            .expect("Given a directory this test created, listing it should always succeed")
            .map(|entry| {
                entry
                    .expect("Given a readable directory, every entry should always be readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        result.sort();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_only_the_document_and_its_metadata_when_persisting_a_record() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());
        let record = sample_raw_document();
        let directory = root.path().join("sec/companyfacts");

        let expected_result = vec![
            "CIK0000320193.json".to_owned(),
            "CIK0000320193.json.meta.json".to_owned(),
        ];

        repository
            .persist(record)
            .await
            .expect("Given an existing writable root, persisting a record should always succeed");
        let mut result: Vec<String> = std::fs::read_dir(directory)
            .expect("Given a persisted record, its directory should always exist")
            .map(|entry| {
                entry
                    .expect("Given a readable directory, every entry should always be readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        result.sort();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unreachable_storage_when_persisting_under_a_missing_root() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let repository = FilesystemRepository::new(&root);
        let io_error = std::fs::metadata(&root)
            .expect_err("Given a root this test never created, inspecting it should always fail");

        let expected_result = Err(WriteError::Backend(BackendError::unreachable_storage(
            format!("Failed to inspect '{}', {io_error}", root.display()),
        )));

        let result = repository.persist(sample_raw_document()).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_create_the_root_when_persisting_under_a_missing_root() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let repository = FilesystemRepository::new(&root);

        let expected_result = false;

        // The write error is the subject of another test. This test checks the disk afterwards.
        let _ = repository.persist(sample_raw_document()).await;
        let result = root.exists();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_operation_when_a_file_stands_where_a_directory_must_be() {
        let root = temporary_root();
        std::fs::write(root.path().join("sec"), b"").expect(
            "Given a writable temp directory, writing a file into it should always succeed",
        );
        let repository = FilesystemRepository::new(root.path());
        let directory = root.path().join("sec/companyfacts");
        let io_error = std::fs::create_dir_all(&directory).expect_err(
            "Given a file named 'sec' in the root, creating a directory below it should always fail",
        );

        let expected_result = Err(WriteError::Backend(BackendError::failed_operation(
            format!(
                "Failed to create directory '{}', {io_error}",
                directory.display()
            ),
        )));

        let result = repository.persist(sample_raw_document()).await;

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
}
