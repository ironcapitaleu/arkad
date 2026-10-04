//! # Read Repository
//!
//! Implements [`ReadRepository`] for [`FilesystemRepository`].

use std::io;

use async_trait::async_trait;
use tokio::fs;

use super::document_key::DocumentKey;
use super::raw_document::RawDocument;
use super::{FilesystemRepository, backend_error};
use crate::error::{BackendError, ReadError};
use crate::traits::repository::ReadRepository;

#[async_trait]
impl ReadRepository for FilesystemRepository {
    type Record = RawDocument;
    type Key = DocumentKey;

    /// Reads the document a key names, together with its metadata.
    ///
    /// Returns `None` if the root exists and holds no document at the key. If the document read
    /// fails, `get` checks the root first, so a bad root always reports the same error.
    ///
    /// `get` does not check the bytes against
    /// [`DocumentMetadata::bytes`](super::DocumentMetadata::bytes) or
    /// [`DocumentMetadata::sha256`](super::DocumentMetadata::sha256).
    ///
    /// # Errors
    ///
    /// Returns a [`ReadError::Backend`] if:
    /// - the root is missing or is not a directory ([`BackendError::UnreachableStorage`]).
    /// - the filesystem denies access to the document or its metadata
    ///   ([`BackendError::UnauthorizedAccess`]).
    /// - the document has no metadata file, the metadata file holds no valid metadata, or the
    ///   filesystem rejects the read for any other reason ([`BackendError::FailedOperation`]).
    async fn get(&self, key: Self::Key) -> Result<Option<Self::Record>, ReadError> {
        let document_path = self.document_path(&key);
        let bytes = match fs::read(&document_path).await {
            Ok(bytes) => bytes,
            Err(error) => {
                self.ensure_root_is_reachable().await?;
                if error.kind() == io::ErrorKind::NotFound {
                    return Ok(None);
                }
                return Err(backend_error("read", &document_path, &error).into());
            }
        };

        let metadata_path = self.metadata_path(&key);
        let metadata_json = fs::read(&metadata_path)
            .await
            .map_err(|error| match error.kind() {
                io::ErrorKind::NotFound => BackendError::failed_operation(format!(
                    "Document '{}' has no metadata file",
                    document_path.display()
                )),
                _ => backend_error("read", &metadata_path, &error),
            })?;
        let metadata = serde_json::from_slice(&metadata_json).map_err(|error| {
            BackendError::failed_operation(format!(
                "Failed to parse '{}', {error}",
                metadata_path.display()
            ))
        })?;

        Ok(Some(RawDocument::new(key, bytes, metadata)))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;
    use crate::implementations::repository::filesystem::DocumentMetadata;
    use crate::tests::fixtures::sample_raw_document::sample_raw_document;

    fn temporary_root() -> TempDir {
        TempDir::new()
            .expect("Given a writable system temp directory, creating a directory in it should always succeed")
    }

    /// Writes the record's document and metadata files by hand, so a test of `get` does not
    /// depend on `persist`.
    fn write_record_files(repository: &FilesystemRepository, record: &RawDocument) {
        let document_path = repository.document_path(record.key());
        let directory = document_path
            .parent()
            .expect("Given a document path under a root, the path should always have a parent");
        std::fs::create_dir_all(directory).expect(
            "Given a writable temp directory, creating subdirectories should always succeed",
        );
        std::fs::write(&document_path, record.bytes())
            .expect("Given an existing directory, writing the document should always succeed");
        let metadata_json = serde_json::to_vec(record.metadata())
            .expect("Given metadata of owned fields, serialization should always succeed");
        std::fs::write(repository.metadata_path(record.key()), metadata_json)
            .expect("Given an existing directory, writing the metadata should always succeed");
    }

    const fn implements_read_repository<T: ReadRepository>() {}
    #[test]
    const fn should_implement_read_repository_for_filesystem_repository() {
        implements_read_repository::<FilesystemRepository>();
    }

    #[tokio::test]
    async fn should_return_the_record_when_the_key_names_a_stored_document() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());
        let record = sample_raw_document();
        write_record_files(&repository, &record);

        let expected_result = Ok(Some(record.clone()));

        let result = repository.get(record.key().clone()).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_the_key_names_no_document() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());

        let expected_result = Ok(None);

        let result = repository.get(sample_raw_document().key().clone()).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unreachable_storage_when_reading_from_a_missing_root() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let repository = FilesystemRepository::new(&root);
        let io_error = std::fs::metadata(&root)
            .expect_err("Given a root this test never created, inspecting it should always fail");

        let expected_result = Err(ReadError::Backend(BackendError::unreachable_storage(
            format!("Failed to inspect '{}', {io_error}", root.display()),
        )));

        let result = repository.get(sample_raw_document().key().clone()).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unreachable_storage_when_reading_from_a_root_that_is_a_file() {
        let parent = temporary_root();
        let root = parent.path().join("file");
        std::fs::write(&root, b"").expect(
            "Given a writable temp directory, writing a file into it should always succeed",
        );
        let repository = FilesystemRepository::new(&root);

        let expected_result = Err(ReadError::Backend(BackendError::unreachable_storage(
            format!("Root '{}' is not a directory", root.display()),
        )));

        let result = repository.get(sample_raw_document().key().clone()).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_parse_the_metadata_when_the_metadata_file_holds_the_stored_format() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());
        let record = sample_raw_document();
        write_record_files(&repository, &record);
        let stored_metadata = r#"{
  "metadata_version": 1,
  "url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json",
  "fetched_at": "2026-09-22T20:05:30Z",
  "http_status": 200,
  "user_agent": "arkad contact@example.com",
  "bytes": 2,
  "sha256": "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
}"#;
        std::fs::write(repository.metadata_path(record.key()), stored_metadata)
            .expect("Given an existing directory, overwriting the metadata should always succeed");

        let expected_result = Ok(Some(record.clone()));

        let result = repository.get(record.key().clone()).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_operation_when_the_document_has_no_metadata_file() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());
        let record = sample_raw_document();
        write_record_files(&repository, &record);
        std::fs::remove_file(repository.metadata_path(record.key())).expect(
            "Given a metadata file written by this test, removing it should always succeed",
        );

        let expected_result = Err(ReadError::Backend(BackendError::failed_operation(format!(
            "Document '{}' has no metadata file",
            repository.document_path(record.key()).display()
        ))));

        let result = repository.get(record.key().clone()).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_operation_when_the_metadata_file_holds_no_valid_metadata() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());
        let record = sample_raw_document();
        write_record_files(&repository, &record);
        let metadata_path = repository.metadata_path(record.key());
        std::fs::write(&metadata_path, b"not json")
            .expect("Given an existing directory, overwriting the metadata should always succeed");
        let parse_error = serde_json::from_slice::<DocumentMetadata>(b"not json")
            .expect_err("Given bytes that are not JSON, parsing them should always fail");

        let expected_result = Err(ReadError::Backend(BackendError::failed_operation(format!(
            "Failed to parse '{}', {parse_error}",
            metadata_path.display()
        ))));

        let result = repository.get(record.key().clone()).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_operation_when_the_key_names_a_directory() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());
        let record = sample_raw_document();
        let document_path = repository.document_path(record.key());
        std::fs::create_dir_all(&document_path).expect(
            "Given a writable temp directory, creating subdirectories should always succeed",
        );
        let io_error = std::fs::read(&document_path).expect_err(
            "Given a path that names a directory, reading it as a file should always fail",
        );

        let expected_result = Err(ReadError::Backend(BackendError::failed_operation(format!(
            "Failed to read '{}', {io_error}",
            document_path.display()
        ))));

        let result = repository.get(record.key().clone()).await;

        assert_eq!(result, expected_result);
    }
}
