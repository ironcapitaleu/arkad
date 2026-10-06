//! # Filesystem Repository
//!
//! Provides [`FilesystemRepository`], the adapter that stores documents as files under a root
//! directory.
//!
//! ## Record and Key
//!
//! [`ReadRepository`](crate::ReadRepository) and [`WriteRepository`](crate::WriteRepository) each
//! leave their associated types to the implementor. This adapter pins `Record` to [`RawDocument`]
//! in both, and `ReadRepository`'s `Key` to [`DocumentKey`].
//!
//! ## Errors
//!
//! The adapter's own code reports a [`FilesystemError`]. The trait methods convert it into a
//! [`BackendError`] at the boundary, so [`FilesystemError`] names no storage error type.
//!
//! ## Modules
//!
//! - [`constants`]: The constants that fix the file layout, such as [`METADATA_SUFFIX`].
//! - [`document_key`]: The [`DocumentKey`] naming one document, and the [`InvalidDocumentKey`] for
//!   a rejected path.
//! - [`document_metadata`]: The [`DocumentMetadata`] describing the fetch behind a document.
//! - [`filesystem_error`]: The [`FilesystemError`] for a failed operation on a path.
//! - [`raw_document`]: The [`RawDocument`] a read returns and a write accepts.

use std::io;
use std::path::{Path, PathBuf};

use crate::error::BackendError;

pub mod constants;
pub mod document_key;
pub mod document_metadata;
pub mod filesystem_error;
pub mod raw_document;
mod read_repository;
mod write_repository;

pub use constants::METADATA_SUFFIX;
pub use document_key::{DocumentKey, InvalidDocumentKey};
pub use document_metadata::DocumentMetadata;
pub use filesystem_error::{FilesystemError, FilesystemErrorReason, FilesystemOperation};
pub use raw_document::RawDocument;

/// Stores documents as files beneath one root directory.
///
/// A [`DocumentKey`] is a document's path under the root. A document lives at the root joined with
/// its key. On Linux and macOS, a root of `/data/arkad` and a key of `sec/CIK0000320193.json` name
/// `/data/arkad/sec/CIK0000320193.json`. On Windows, the same key under a root of `C:\data\arkad`
/// names `C:\data\arkad\sec\CIK0000320193.json`. The join uses the separator of the platform the
/// store runs on. [`FilesystemRepository::document_path`] performs that join.
///
/// Each document has a metadata file beside it. The metadata file name is the document's file name
/// with [`METADATA_SUFFIX`] appended, so `sec/CIK0000320193.json` keeps its metadata in
/// `sec/CIK0000320193.json<METADATA_SUFFIX>`. A key that ends in [`METADATA_SUFFIX`] names the
/// metadata file of another key, so one store must not hold both keys.
///
/// A write first writes both files to synced temporary files in the same directory. It then
/// renames the metadata over its target, and the document over its target. A reader sees the old
/// file or the new file, never a partial one. A document on disk always has its metadata. Between
/// the two renames, a reader can see new metadata beside the old document. If the document rename
/// fails, or the caller drops the write between the renames, the new metadata stays beside the old
/// document until the next successful write under that key. If the caller drops the write before
/// the renames, its temporary files stay in the directory.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilesystemRepository {
    root: PathBuf,
}

impl FilesystemRepository {
    /// Creates a new [`FilesystemRepository`] rooted at the given directory.
    ///
    /// The constructor checks nothing and touches no disk. Whether the root exists, is a
    /// directory, and allows reads or writes can change at any time, and it can differ for each
    /// document under the root. Each operation reports those failures for the document it touches.
    ///
    /// Pass an absolute root. A relative root resolves against the process's working directory,
    /// so the store would move with it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    ///
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
    ///
    /// let repository = FilesystemRepository::new("/data/arkad");
    ///
    /// let expected_result = Path::new("/data/arkad");
    ///
    /// let result = repository.root();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the root directory every key resolves under.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the path on disk of the document a key names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    ///
    /// use storage::implementations::repository::filesystem::{DocumentKey, FilesystemRepository};
    ///
    /// let repository = FilesystemRepository::new("/data/arkad");
    /// let key = DocumentKey::new("sec/CIK0000320193.json")
    ///     .expect("Given a valid relative path, the key should always build");
    ///
    /// let expected_result = Path::new("/data/arkad/sec/CIK0000320193.json");
    ///
    /// let result = repository.document_path(&key);
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn document_path(&self, key: &DocumentKey) -> PathBuf {
        self.root.join(key.as_path())
    }

    /// Returns the path on disk of the metadata file beside a key's document.
    fn metadata_path(&self, key: &DocumentKey) -> PathBuf {
        let mut path = self.document_path(key).into_os_string();
        path.push(METADATA_SUFFIX);
        PathBuf::from(path)
    }

    /// Checks that the root exists and is a directory.
    ///
    /// # Errors
    ///
    /// Returns a [`FilesystemError`] with:
    /// - [`FilesystemErrorReason::MissingRoot`] if the root does not exist, or a file stands in
    ///   its path.
    /// - [`FilesystemErrorReason::RootIsNotADirectory`] if the root is a file.
    /// - [`FilesystemErrorReason::FailedIo`] if the filesystem cannot inspect the root.
    async fn ensure_root_is_reachable(&self) -> Result<(), FilesystemError> {
        match tokio::fs::metadata(&self.root).await {
            Ok(metadata) if metadata.is_dir() => Ok(()),
            Ok(_) => Err(FilesystemError::new(
                FilesystemErrorReason::RootIsNotADirectory,
                &self.root,
            )),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) =>
            {
                Err(FilesystemError::new(
                    FilesystemErrorReason::MissingRoot,
                    &self.root,
                ))
            }
            Err(error) => Err(FilesystemError::from_io(
                FilesystemOperation::Inspect,
                &self.root,
                &error,
            )),
        }
    }
}

impl From<FilesystemError> for BackendError {
    /// Converts a [`FilesystemError`] into the [`BackendError`] variant for its reason.
    ///
    /// A missing root, a root that is not a directory, and a path that vanished become
    /// [`BackendError::UnreachableStorage`]. A denied permission becomes
    /// [`BackendError::UnauthorizedAccess`]. Every other reason becomes
    /// [`BackendError::FailedOperation`]. The [`FilesystemError`] message becomes the reason.
    fn from(error: FilesystemError) -> Self {
        let reason = error.to_string();
        match error.reason {
            FilesystemErrorReason::MissingRoot
            | FilesystemErrorReason::RootIsNotADirectory
            | FilesystemErrorReason::FailedIo {
                kind: io::ErrorKind::NotFound,
                ..
            } => Self::unreachable_storage(reason),
            FilesystemErrorReason::FailedIo {
                kind: io::ErrorKind::PermissionDenied,
                ..
            } => Self::unauthorized_access(reason),
            FilesystemErrorReason::MissingMetadataFile
            | FilesystemErrorReason::InvalidMetadata { .. }
            | FilesystemErrorReason::UnserializableMetadata
            | FilesystemErrorReason::FailedIo { .. } => Self::failed_operation(reason),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    use super::*;
    use crate::traits::repository::ReadWriteRepository;

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filesystem_repository() {
        implements_auto_traits::<FilesystemRepository>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_filesystem_repository() {
        implements_send::<FilesystemRepository>();
    }

    #[test]
    const fn should_implement_sync_for_filesystem_repository() {
        implements_sync::<FilesystemRepository>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_filesystem_repository() {
        implements_debug::<FilesystemRepository>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_filesystem_repository() {
        implements_clone::<FilesystemRepository>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_filesystem_repository() {
        implements_hash::<FilesystemRepository>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_filesystem_repository() {
        implements_eq::<FilesystemRepository>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_filesystem_repository() {
        implements_ord::<FilesystemRepository>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_filesystem_repository() {
        implements_sized::<FilesystemRepository>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_filesystem_repository() {
        implements_partial_eq::<FilesystemRepository>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_filesystem_repository() {
        implements_partial_ord::<FilesystemRepository>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_filesystem_repository() {
        implements_unpin::<FilesystemRepository>();
    }

    const fn implements_read_write_repository<T: ReadWriteRepository>() {}
    #[test]
    const fn should_implement_read_write_repository_for_filesystem_repository() {
        implements_read_write_repository::<FilesystemRepository>();
    }

    #[test]
    fn should_hold_the_root_when_building_a_repository() {
        let root = "/data/arkad";

        let expected_result = Path::new("/data/arkad");

        let repository = FilesystemRepository::new(root);
        let result = repository.root();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_join_the_root_and_the_key_when_building_a_document_path() {
        let repository = FilesystemRepository::new("/data/arkad");
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        let expected_result = Path::new("/data/arkad/sec/companyfacts/CIK0000320193.json");

        let result = repository.document_path(&key);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_append_the_metadata_suffix_for_a_document_key() {
        let repository = FilesystemRepository::new("/data/arkad");
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        let expected_result =
            Path::new("/data/arkad/sec/companyfacts/CIK0000320193.json.meta.json");

        let result = repository.metadata_path(&key);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unreachable_storage_when_converting_a_missing_root() {
        let error = FilesystemError::new(FilesystemErrorReason::MissingRoot, "/data/arkad");

        let expected_result = BackendError::unreachable_storage(
            "[FilesystemError] Filesystem operation failed, \
            Reason: 'Root directory does not exist', Path: '/data/arkad'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unreachable_storage_when_converting_a_root_that_is_not_a_directory() {
        let error = FilesystemError::new(FilesystemErrorReason::RootIsNotADirectory, "/data/arkad");

        let expected_result = BackendError::unreachable_storage(
            "[FilesystemError] Filesystem operation failed, \
            Reason: 'Root is not a directory', Path: '/data/arkad'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unreachable_storage_when_converting_a_path_that_vanished() {
        let io_error = io::Error::from(io::ErrorKind::NotFound);
        let error = FilesystemError::from_io(
            FilesystemOperation::CreateFile,
            "/data/arkad/x.json",
            &io_error,
        );

        let expected_result = BackendError::unreachable_storage(
            "[FilesystemError] Filesystem operation failed, \
            Reason: 'Failed to create file, entity not found', Path: '/data/arkad/x.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unauthorized_access_when_converting_a_denied_permission() {
        let io_error = io::Error::from(io::ErrorKind::PermissionDenied);
        let error =
            FilesystemError::from_io(FilesystemOperation::Read, "/data/arkad/x.json", &io_error);

        let expected_result = BackendError::unauthorized_access(
            "[FilesystemError] Filesystem operation failed, \
            Reason: 'Failed to read, permission denied', Path: '/data/arkad/x.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_operation_when_converting_a_full_disk() {
        let io_error = io::Error::from(io::ErrorKind::StorageFull);
        let error =
            FilesystemError::from_io(FilesystemOperation::Write, "/data/arkad/x.json", &io_error);

        let expected_result = BackendError::failed_operation(
            "[FilesystemError] Filesystem operation failed, \
            Reason: 'Failed to write, no storage space', Path: '/data/arkad/x.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_operation_when_converting_a_missing_metadata_file() {
        let error = FilesystemError::new(
            FilesystemErrorReason::MissingMetadataFile,
            "/data/arkad/x.json.meta.json",
        );

        let expected_result = BackendError::failed_operation(
            "[FilesystemError] Filesystem operation failed, \
            Reason: 'Metadata file does not exist', Path: '/data/arkad/x.json.meta.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_accept_the_root_when_the_root_is_an_existing_directory() {
        let root = temporary_root();
        let repository = FilesystemRepository::new(root.path());

        let expected_result = Ok(());

        let result = repository.ensure_root_is_reachable().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_root_when_the_root_does_not_exist() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let repository = FilesystemRepository::new(&root);

        let expected_result = Err(FilesystemError::new(
            FilesystemErrorReason::MissingRoot,
            &root,
        ));

        let result = repository.ensure_root_is_reachable().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_root_when_the_root_has_a_file_in_its_path() {
        let parent = temporary_root();
        let file = parent.path().join("file");
        std::fs::write(&file, b"").expect(
            "Given a writable temp directory, writing a file into it should always succeed",
        );
        let root = file.join("data");
        let repository = FilesystemRepository::new(&root);

        let expected_result = Err(FilesystemError::new(
            FilesystemErrorReason::MissingRoot,
            &root,
        ));

        let result = repository.ensure_root_is_reachable().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_root_is_not_a_directory_when_the_root_is_a_file() {
        let parent = temporary_root();
        let root = parent.path().join("file");
        std::fs::write(&root, b"").expect(
            "Given a writable temp directory, writing a file into it should always succeed",
        );
        let repository = FilesystemRepository::new(&root);

        let expected_result = Err(FilesystemError::new(
            FilesystemErrorReason::RootIsNotADirectory,
            &root,
        ));

        let result = repository.ensure_root_is_reachable().await;

        assert_eq!(result, expected_result);
    }
}
