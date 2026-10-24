//! # Read Repository
//!
//! Implements [`ReadRepository`] for [`FilesystemRepository`].

use std::io;
use std::path::Path;

use async_trait::async_trait;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tokio::fs;

use crate::error::ReadError;
use crate::implementations::repository::filesystem::error::{
    FailedFilesystemOperation, FailedFilesystemOperationReason, FilesystemOperation,
};
use crate::implementations::repository::filesystem::shared::document_key::DocumentKey;
use crate::implementations::repository::filesystem::shared::document_metadata::DocumentMetadata;
use crate::implementations::repository::filesystem::traits::access::CanRead;
use crate::implementations::repository::filesystem::traits::document::Document;
use crate::implementations::repository::filesystem::{FilesystemRepository, check_content};
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
    /// `get` reads `kind` and `kind_version` from the metadata file before the rest of it. A file
    /// of another kind or kind version fails with both named, even if its other fields do not fit
    /// the context of `D`.
    ///
    /// `get` rejects a key that ends in
    /// [`METADATA_SUFFIX`](crate::implementations::repository::filesystem::METADATA_SUFFIX) before
    /// it reads a file. After the read, `get` checks the length of the bytes against
    /// [`DocumentMetadata::size_in_bytes`], then their SHA-256 digest against
    /// [`DocumentMetadata::sha256`]. A torn write or a corrupt file fails one of the two checks.
    /// `get` does not compare [`DocumentMetadata::metadata_version`] with
    /// [`METADATA_VERSION`](crate::implementations::repository::filesystem::METADATA_VERSION).
    ///
    /// # Errors
    ///
    /// Returns a [`ReadError::Backend`] that holds the [`FailedFilesystemOperation`] message if:
    /// - the root does not exist, is not a directory, or has a file in its path
    ///   ([`BackendError::UnreachableStorage`](crate::BackendError::UnreachableStorage)).
    /// - the filesystem denies access to the document or its metadata
    ///   ([`BackendError::UnauthorizedAccess`](crate::BackendError::UnauthorizedAccess)).
    /// - the key ends in the metadata suffix, the document has no metadata file, the metadata file
    ///   holds no valid metadata, the metadata holds another kind or kind version than `D`, the
    ///   bytes have another size or digest than the metadata names, or the filesystem rejects the
    ///   read for any other reason
    ///   ([`BackendError::FailedOperation`](crate::BackendError::FailedOperation)).
    async fn get(&self, key: Self::Key) -> Result<Option<Self::Record>, ReadError> {
        self.read_record(key)
            .await
            .map_err(|error| ReadError::Backend(error.into()))
    }
}

impl<A: CanRead, D: Document> FilesystemRepository<A, D> {
    /// Reads the document at the key and the metadata file beside it, then checks one against the
    /// other.
    async fn read_record(&self, key: DocumentKey) -> Result<Option<D>, FailedFilesystemOperation> {
        self.check_key(&key)?;
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
                return Err(FailedFilesystemOperation::from_io(
                    FilesystemOperation::Read,
                    &document_path,
                    &error,
                ));
            }
        };

        let metadata = self.read_metadata(&key).await?;
        check_content(&bytes, &metadata)
            .map_err(|reason| FailedFilesystemOperation::new(reason, &document_path))?;
        Ok(Some(D::from_parts(key, bytes, metadata)))
    }

    /// Reads the metadata file beside the key's document, checks its kind, then parses it.
    async fn read_metadata(
        &self,
        key: &DocumentKey,
    ) -> Result<DocumentMetadata<D::Context>, FailedFilesystemOperation> {
        let metadata_path = self.metadata_path(key);
        let metadata_json = fs::read(&metadata_path).await.map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                FailedFilesystemOperation::new(
                    FailedFilesystemOperationReason::MissingMetadataFile,
                    &metadata_path,
                )
            } else {
                FailedFilesystemOperation::from_io(
                    FilesystemOperation::Read,
                    &metadata_path,
                    &error,
                )
            }
        })?;

        let stored: StoredKind = parse_metadata(&metadata_json, &metadata_path)?;
        let expected_kind = D::KIND.to_string();
        if stored.kind != expected_kind || stored.kind_version != D::KIND_VERSION {
            let reason = FailedFilesystemOperationReason::UnexpectedMetadataKind {
                found_kind: stored.kind,
                found_kind_version: stored.kind_version,
                expected_kind,
                expected_kind_version: D::KIND_VERSION,
            };
            return Err(FailedFilesystemOperation::new(reason, &metadata_path));
        }
        parse_metadata(&metadata_json, &metadata_path)
    }
}

/// The `kind` and `kind_version` of a metadata file, which a read checks before it parses the rest.
#[derive(Deserialize)]
struct StoredKind {
    kind: String,
    kind_version: u32,
}

/// Parses a metadata file, and reports where the parser stopped if the file does not fit `T`.
fn parse_metadata<T: DeserializeOwned>(
    metadata_json: &[u8],
    metadata_path: &Path,
) -> Result<T, FailedFilesystemOperation> {
    serde_json::from_slice(metadata_json).map_err(|error| {
        let reason = FailedFilesystemOperationReason::InvalidMetadata {
            line: error.line(),
            column: error.column(),
        };
        FailedFilesystemOperation::new(reason, metadata_path)
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
    use crate::shared::Sha256Digest;
    use crate::tests::fixtures::sample_document::{SampleDocument, sample_document};

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

    /// Converts a [`FailedFilesystemOperation`] into the [`ReadError`] that `get` returns for it.
    fn read_error(
        reason: FailedFilesystemOperationReason,
        path: impl Into<std::path::PathBuf>,
    ) -> ReadError {
        ReadError::Backend(BackendError::from(FailedFilesystemOperation::new(
            reason, path,
        )))
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

        let expected_result = Err(read_error(
            FailedFilesystemOperationReason::MissingRoot,
            &root,
        ));

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
            FailedFilesystemOperationReason::RootIsNotADirectory,
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
  "kind": "sample_document",
  "kind_version": 1,
  "media_type": "application/json",
  "bytes": 2,
  "sha256": "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
  "produced_at": "2026-09-22T20:05:30Z",
  "execution_id": "0b6f3c1e-5d2a-4f7b-9c3e-8a1d2b4c6e8f",
  "producer": { "name": "Sample State", "crate_version": "0.1.0", "commit": "63c1f59" },
  "inputs": [{
    "key": "sec/companyfacts/CIK0000320193.json",
    "sha256": "73a86c6aedc31f77cac2ea4df5f80f0b73a86c6aedc31f77cac2ea4df5f80f0b"
  }],
  "context": { "url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json" }
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
            FailedFilesystemOperationReason::MissingMetadataFile,
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
            FailedFilesystemOperationReason::InvalidMetadata { line: 1, column: 2 },
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
            FailedFilesystemOperationReason::FailedIo {
                operation: FilesystemOperation::Read,
                kind: io::ErrorKind::IsADirectory,
            },
            &document_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    /// Writes the sample record's files, then sets the given fields of its stored metadata.
    fn write_record_files_with_metadata_fields(
        repository: &FilesystemRepository<Read, SampleDocument>,
        fields: &[(&str, serde_json::Value)],
    ) {
        let sample_document = sample_document();
        write_record_files(repository, &sample_document);
        let mut stored = serde_json::to_value(sample_document.metadata())
            .expect("Given metadata of owned fields, serialization should always succeed");
        for (field, value) in fields {
            stored[*field] = value.clone();
        }
        let metadata_json = serde_json::to_vec(&stored)
            .expect("Given a JSON value, serialization should always succeed");
        std::fs::write(
            repository.metadata_path(sample_document.key()),
            metadata_json,
        )
        .expect("Given an existing directory, overwriting the metadata should always succeed");
    }

    #[tokio::test]
    async fn should_return_unexpected_metadata_kind_when_the_metadata_holds_another_kind() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let key = sample_document().key().clone();
        let metadata_path = repository.metadata_path(&key);
        write_record_files_with_metadata_fields(&repository, &[("kind", "other_document".into())]);

        let expected_result = Err(read_error(
            FailedFilesystemOperationReason::UnexpectedMetadataKind {
                found_kind: "other_document".to_owned(),
                found_kind_version: 1,
                expected_kind: "sample_document".to_owned(),
                expected_kind_version: 1,
            },
            &metadata_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unexpected_metadata_kind_when_the_metadata_holds_another_kind_version() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let key = sample_document().key().clone();
        let metadata_path = repository.metadata_path(&key);
        write_record_files_with_metadata_fields(&repository, &[("kind_version", 2.into())]);

        let expected_result = Err(read_error(
            FailedFilesystemOperationReason::UnexpectedMetadataKind {
                found_kind: "sample_document".to_owned(),
                found_kind_version: 2,
                expected_kind: "sample_document".to_owned(),
                expected_kind_version: 1,
            },
            &metadata_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_report_the_kind_before_the_context_when_another_kind_has_another_context() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let key = sample_document().key().clone();
        let metadata_path = repository.metadata_path(&key);
        write_record_files_with_metadata_fields(
            &repository,
            &[
                ("kind", "other_document".into()),
                ("context", serde_json::json!({ "other_field": 1 })),
            ],
        );

        let expected_result = Err(read_error(
            FailedFilesystemOperationReason::UnexpectedMetadataKind {
                found_kind: "other_document".to_owned(),
                found_kind_version: 1,
                expected_kind: "sample_document".to_owned(),
                expected_kind_version: 1,
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
        std::fs::write(&metadata_path, br#"{"kind_version":1}"#)
            .expect("Given an existing directory, overwriting the metadata should always succeed");

        let expected_result = Err(read_error(
            FailedFilesystemOperationReason::InvalidMetadata {
                line: 1,
                column: 18,
            },
            &metadata_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unexpected_document_size_when_the_document_is_longer_than_its_metadata()
    {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        let document_path = repository.document_path(&key);
        write_record_files(&repository, &sample_document);
        std::fs::write(&document_path, b"{ }")
            .expect("Given an existing directory, overwriting the document should always succeed");

        let expected_result = Err(read_error(
            FailedFilesystemOperationReason::UnexpectedDocumentSize {
                found_size: 3,
                expected_size: 2,
            },
            &document_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_unexpected_document_digest_when_the_document_differs_from_its_metadata()
    {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        let document_path = repository.document_path(&key);
        write_record_files(&repository, &sample_document);
        std::fs::write(&document_path, b"[]")
            .expect("Given an existing directory, overwriting the document should always succeed");

        let expected_result = Err(read_error(
            FailedFilesystemOperationReason::UnexpectedDocumentDigest {
                found_digest: Sha256Digest::of(b"[]"),
                expected_digest: sample_document.metadata().sha256().to_owned(),
            },
            &document_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_key_suffix_when_reading_a_key_that_ends_in_the_metadata_suffix()
    {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json.meta.json")
            .expect("Given a valid relative path, the key should always build");
        let document_path = repository.document_path(&key);
        write_record_files(&repository, &sample_document);

        let expected_result = Err(read_error(
            FailedFilesystemOperationReason::InvalidKeySuffix,
            &document_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_metadata_when_the_stored_sha256_is_not_a_digest() {
        let root = temporary_root();
        let repository = repository_at(root.path());
        let sample_document = sample_document();
        let key = sample_document.key().clone();
        let metadata_path = repository.metadata_path(&key);
        write_record_files(&repository, &sample_document);
        let stored_metadata = r#"{
  "metadata_version": 1,
  "kind": "sample_document",
  "kind_version": 1,
  "media_type": "application/json",
  "bytes": 2,
  "sha256": "NOT-HEX",
  "produced_at": "2026-09-22T20:05:30Z",
  "execution_id": "0b6f3c1e-5d2a-4f7b-9c3e-8a1d2b4c6e8f",
  "producer": { "name": "Sample State", "crate_version": "0.1.0", "commit": "63c1f59" },
  "inputs": [],
  "context": { "url": "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json" }
}"#;
        std::fs::write(&metadata_path, stored_metadata)
            .expect("Given an existing directory, overwriting the metadata should always succeed");

        let expected_result = Err(read_error(
            FailedFilesystemOperationReason::InvalidMetadata {
                line: 7,
                column: 21,
            },
            &metadata_path,
        ));

        let result = repository.get(key).await;

        assert_eq!(result, expected_result);
    }
}
