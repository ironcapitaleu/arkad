//! # Read Repository
//!
//! Implements [`ReadRepository`] for [`FilesystemRepository`].

use std::io;
use std::path::Path;

use async_trait::async_trait;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tokio::fs;

use super::FilesystemRepository;
use super::access::CanRead;
use super::filesystem_error::{FilesystemError, FilesystemErrorReason, FilesystemOperation};
use crate::error::ReadError;
use crate::shared::document_key::DocumentKey;
use crate::shared::document_metadata::DocumentMetadata;
use crate::traits::document::Document;
use crate::traits::metadata_context::MetadataContext;
use crate::traits::repository::ReadRepository;

#[async_trait]
impl<A: CanRead, D: Document> ReadRepository for FilesystemRepository<A, D> {
    type Record = D;
    type Key = DocumentKey;

    /// Reads the document a key identifies, together with its metadata.
    ///
    /// Returns `None` if the root exists and holds no document at the key. A key whose path runs
    /// through a stored file also returns `None`. If the document read fails, `get` checks the
    /// root first, so a bad root always reports the same error.
    ///
    /// `get` reads the `kind` of the metadata file before the rest of it. A file of another kind
    /// fails with its kind named, even if its other fields do not fit the context of `D`.
    ///
    /// `get` does not check the bytes against [`DocumentMetadata::bytes`] or
    /// [`DocumentMetadata::sha256`], and it does not compare [`DocumentMetadata::metadata_version`]
    /// with [`METADATA_VERSION`](crate::METADATA_VERSION).
    ///
    /// # Errors
    ///
    /// Returns a [`ReadError::Backend`] that holds the [`FilesystemError`] message if:
    /// - the root does not exist, is not a directory, or has a file in its path
    ///   ([`BackendError::UnreachableStorage`](crate::BackendError::UnreachableStorage)).
    /// - the filesystem denies access to the document or its metadata
    ///   ([`BackendError::UnauthorizedAccess`](crate::BackendError::UnauthorizedAccess)).
    /// - the document has no metadata file, or the metadata file holds no valid metadata
    ///   ([`BackendError::FailedOperation`](crate::BackendError::FailedOperation)).
    /// - the metadata file holds another kind than the [`MetadataContext::KIND`] of `D`
    ///   ([`BackendError::FailedOperation`](crate::BackendError::FailedOperation)).
    /// - the filesystem rejects the read for any other reason
    ///   ([`BackendError::FailedOperation`](crate::BackendError::FailedOperation)).
    async fn get(&self, key: Self::Key) -> Result<Option<Self::Record>, ReadError> {
        self.read_record(key)
            .await
            .map_err(|error| ReadError::Backend(error.into()))
    }
}

impl<A: CanRead, D: Document> FilesystemRepository<A, D> {
    /// Reads the document at the key and the metadata file beside it.
    async fn read_record(&self, key: DocumentKey) -> Result<Option<D>, FilesystemError> {
        let document_path = self.document_path(&key);
        let bytes = match fs::read(&document_path).await {
            Ok(bytes) => bytes,
            Err(error) => {
                self.ensure_root_is_reachable().await?;
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) {
                    return Ok(None);
                }
                return Err(FilesystemError::from_io(
                    FilesystemOperation::Read,
                    &document_path,
                    &error,
                ));
            }
        };

        let metadata = self.read_metadata(&key).await?;
        Ok(Some(D::from_parts(key, bytes, metadata)))
    }

    /// Reads the metadata file beside the key's document, checks its kind, then parses it.
    async fn read_metadata(
        &self,
        key: &DocumentKey,
    ) -> Result<DocumentMetadata<D::Context>, FilesystemError> {
        let metadata_path = self.metadata_path(key);
        let metadata_json = fs::read(&metadata_path).await.map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                FilesystemError::new(FilesystemErrorReason::MissingMetadataFile, &metadata_path)
            } else {
                FilesystemError::from_io(FilesystemOperation::Read, &metadata_path, &error)
            }
        })?;

        let stored: StoredKind = parse_metadata(&metadata_json, &metadata_path)?;
        if stored.kind != D::Context::KIND {
            let reason = FilesystemErrorReason::UnexpectedMetadataKind {
                found: stored.kind,
                expected: D::Context::KIND.to_owned(),
            };
            return Err(FilesystemError::new(reason, &metadata_path));
        }
        parse_metadata(&metadata_json, &metadata_path)
    }
}

/// The `kind` field of a metadata file, which a read checks before it parses the rest.
#[derive(Deserialize)]
struct StoredKind {
    kind: String,
}

/// Parses a metadata file, and reports where the parser stopped if the file does not fit `T`.
fn parse_metadata<T: DeserializeOwned>(
    metadata_json: &[u8],
    metadata_path: &Path,
) -> Result<T, FilesystemError> {
    serde_json::from_slice(metadata_json).map_err(|error| {
        let reason = FilesystemErrorReason::InvalidMetadata {
            line: error.line(),
            column: error.column(),
        };
        FilesystemError::new(reason, metadata_path)
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;
    use crate::error::BackendError;
    use crate::implementations::repository::filesystem::{Read, ReadWrite};
    use crate::tests::fixtures::sample_document::{SampleContext, SampleDocument, sample_document};

    /// Builds a repository with read access under the given root.
    fn repository_at(root: impl Into<PathBuf>) -> FilesystemRepository<Read, SampleDocument> {
        FilesystemRepository::builder()
            .reader()
            .document_type::<SampleDocument>()
            .root(root)
            .build()
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    /// Writes the record's document and metadata files by hand, so a test of `get` does not
    /// depend on `persist`.
    fn write_record_files(
        repository: &FilesystemRepository<Read, SampleDocument>,
        record: &SampleDocument,
    ) {
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

    /// Converts a [`FilesystemError`] into the [`ReadError`] that `get` returns for it.
    fn read_error(reason: FilesystemErrorReason, path: impl Into<std::path::PathBuf>) -> ReadError {
        ReadError::Backend(BackendError::from(FilesystemError::new(reason, path)))
    }

    const fn implements_read_repository<T: ReadRepository>() {}
    #[test]
    const fn should_implement_read_repository_for_filesystem_repository_with_read_access() {
        implements_read_repository::<FilesystemRepository<Read, SampleDocument>>();
    }

    #[test]
    const fn should_implement_read_repository_for_filesystem_repository_with_read_write_access() {
        implements_read_repository::<FilesystemRepository<ReadWrite, SampleDocument>>();
    }

    #[tokio::test]
    async fn should_return_the_record_when_the_key_names_a_stored_document() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        write_record_files(&repository, &sample_document);

        let expected_result = Ok(Some(sample_document));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_the_key_names_no_document() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();

        let expected_result = Ok(None);

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_a_key_component_names_a_stored_file() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        std::fs::write(root.path().join("sec"), b"").expect(
            "Given a writable temp directory, writing a file into it should always succeed",
        );

        let expected_result = Ok(None);

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_root_when_reading_from_a_root_that_does_not_exist() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let repository = repository_at(&root);
        let sample_document = sample_document();
        let key = sample_document.key().clone();

        let expected_result = Err(read_error(FilesystemErrorReason::MissingRoot, &root));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_root_is_not_a_directory_when_reading_from_a_root_that_is_a_file() {
        let parent = temporary_root();
        let root = parent.path().join("file");
        std::fs::write(&root, b"").expect(
            "Given a writable temp directory, writing a file into it should always succeed",
        );
        let repository = repository_at(&root);
        let sample_document = sample_document();
        let key = sample_document.key().clone();

        let expected_result = Err(read_error(
            FilesystemErrorReason::RootIsNotADirectory,
            &root,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_parse_the_metadata_when_the_metadata_file_holds_the_stored_format() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        let metadata_path = repository.metadata_path(&key);
        write_record_files(&repository, &sample_document);
        let stored_metadata = r#"{
  "metadata_version": 1,
  "kind": "sample_document.v1",
  "media_type": "application/json",
  "bytes": 2,
  "sha256": "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
  "produced_at": "2026-09-22T20:05:30Z",
  "execution_id": "0b6f3c1e-5d2a-4f7b-9c3e-8a1d2b4c6e8f",
  "producer": { "name": "Sample Step", "crate_version": "0.1.0", "commit": "63c1f59" },
  "inputs": [],
  "context": { "source": "a hardcoded test value" }
}"#;
        std::fs::write(&metadata_path, stored_metadata)
            .expect("Given an existing directory, overwriting the metadata should always succeed");

        let expected_result = Ok(Some(sample_document));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_metadata_file_when_the_document_has_no_metadata_file() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        let metadata_path = repository.metadata_path(&key);
        write_record_files(&repository, &sample_document);
        std::fs::remove_file(&metadata_path).expect(
            "Given a metadata file written by this test, removing it should always succeed",
        );

        let expected_result = Err(read_error(
            FilesystemErrorReason::MissingMetadataFile,
            &metadata_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_metadata_when_the_metadata_file_holds_no_valid_metadata() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        let metadata_path = repository.metadata_path(&key);
        write_record_files(&repository, &sample_document);
        std::fs::write(&metadata_path, b"not json")
            .expect("Given an existing directory, overwriting the metadata should always succeed");

        let expected_result = Err(read_error(
            FilesystemErrorReason::InvalidMetadata { line: 1, column: 2 },
            &metadata_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_read_when_the_key_names_a_directory() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        let document_path = repository.document_path(&key);
        std::fs::create_dir_all(&document_path).expect(
            "Given a writable temp directory, creating subdirectories should always succeed",
        );

        let expected_result = Err(read_error(
            FilesystemErrorReason::FailedIo {
                operation: FilesystemOperation::Read,
                kind: io::ErrorKind::IsADirectory,
            },
            &document_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unexpected_metadata_kind_when_the_metadata_file_holds_another_kind() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        let metadata_path = repository.metadata_path(&key);
        write_record_files(&repository, &sample_document);
        let mut other_kind = serde_json::to_value(sample_document.metadata())
            .expect("Given the sample metadata, serialization should always succeed");
        other_kind["kind"] = "other_kind.v1".into();
        std::fs::write(&metadata_path, other_kind.to_string())
            .expect("Given an existing directory, overwriting the metadata should always succeed");

        let expected_result = Err(read_error(
            FilesystemErrorReason::UnexpectedMetadataKind {
                found: "other_kind.v1".to_owned(),
                expected: SampleContext::KIND.to_owned(),
            },
            &metadata_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unexpected_metadata_kind_when_another_kind_has_other_fields() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        let metadata_path = repository.metadata_path(&key);
        write_record_files(&repository, &sample_document);
        std::fs::write(
            &metadata_path,
            r#"{"kind": "other_kind.v1", "context": {}}"#,
        )
        .expect("Given an existing directory, overwriting the metadata should always succeed");

        let expected_result = Err(read_error(
            FilesystemErrorReason::UnexpectedMetadataKind {
                found: "other_kind.v1".to_owned(),
                expected: SampleContext::KIND.to_owned(),
            },
            &metadata_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_metadata_when_the_metadata_file_has_no_kind() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        let metadata_path = repository.metadata_path(&key);
        write_record_files(&repository, &sample_document);
        std::fs::write(&metadata_path, b"{}")
            .expect("Given an existing directory, overwriting the metadata should always succeed");

        let expected_result = Err(read_error(
            FilesystemErrorReason::InvalidMetadata { line: 1, column: 2 },
            &metadata_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_metadata_when_the_kind_matches_but_other_fields_are_missing() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        let metadata_path = repository.metadata_path(&key);
        write_record_files(&repository, &sample_document);
        std::fs::write(&metadata_path, br#"{"kind":"sample_document.v1"}"#)
            .expect("Given an existing directory, overwriting the metadata should always succeed");

        let expected_result = Err(read_error(
            FilesystemErrorReason::InvalidMetadata {
                line: 1,
                column: 29,
            },
            &metadata_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }
}
