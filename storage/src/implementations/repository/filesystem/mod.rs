//! # Filesystem Repository
//!
//! Provides [`FilesystemRepository`], which stores documents as files under a root
//! directory, and the [`FilesystemRepositoryBuilder`] that builds it.
//!
//! ## Record and Key
//!
//! [`ReadRepository`](crate::ReadRepository) and [`WriteRepository`](crate::WriteRepository) each
//! leave their associated types to the implementor. This repository pins `Record` to [`RawDocument`]
//! in both, and `ReadRepository`'s `Key` to [`DocumentKey`].
//!
//! ## Errors
//!
//! [`DocumentKey::new`] reports an [`InvalidDocumentKey`] for a path the store rejects. The
//! repository takes only a valid [`DocumentKey`], so its own code reports a
//! [`FailedFilesystemOperation`].
//!
//! ## Modules
//!
//! - [`error`]: The [`FailedFilesystemOperation`] for an operation on a path that did not
//!   complete.
//! - [`implementations`]: The [`FilesystemRepositoryBuilder`], the constants that fix the file
//!   layout, such as [`METADATA_SUFFIX`], and the port implementations.
//! - [`shared`]: The value types that more than one module uses, such as the [`DocumentKey`] and
//!   the [`RawDocument`].
//! - [`traits`]: The access markers, the traits that connect them to the ports, and the
//!   [`Document`] trait every stored document kind implements.

use std::io;
use std::path::{Path, PathBuf};

use crate::error::BackendError;

pub mod error;
pub mod implementations;
pub mod shared;
pub mod traits;

pub use error::{FailedFilesystemOperation, FailedFilesystemOperationReason, FilesystemOperation};
pub use implementations::builder::{FilesystemRepositoryBuilder, NoAccess, NoRoot};
pub use implementations::constants::METADATA_SUFFIX;
pub use shared::document_key::{DocumentKey, InvalidDocumentKey};
pub use shared::document_metadata::{
    DocumentMetadata, DocumentMetadataBuilder, InputDocument, METADATA_VERSION, Producer,
};
pub use shared::legacy_document_metadata::LegacyDocumentMetadata;
pub use shared::raw_document::RawDocument;
pub use traits::access::{Access, CanRead, CanWrite, Read, ReadWrite, Write};
pub use traits::document::Document;

/// Stores documents as files beneath one root directory, with the access the marker `A` allows.
///
/// [`FilesystemRepository::builder`] starts the builder that builds it. `A` is [`Read`], [`Write`],
/// or [`ReadWrite`].
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
pub struct FilesystemRepository<A> {
    root: PathBuf,
    access: A,
}

impl FilesystemRepository<NoAccess> {
    /// Starts a [`FilesystemRepositoryBuilder`], which is the only way to build a
    /// [`FilesystemRepository`].
    ///
    /// The impl uses `NoAccess` so that a call needs no type annotation. No built repository has
    /// `NoAccess`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    ///
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
    ///
    /// let repository = FilesystemRepository::builder()
    ///     .reader()
    ///     .root("/data/arkad")
    ///     .build();
    ///
    /// let expected_result = Path::new("/data/arkad");
    ///
    /// let result = repository.root();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn builder() -> FilesystemRepositoryBuilder<NoAccess, NoRoot> {
        FilesystemRepositoryBuilder::new()
    }
}

impl<A> FilesystemRepository<A> {
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
    /// use storage::implementations::repository::filesystem::{
    ///     DocumentKey, FilesystemRepository,
    /// };
    ///
    /// let repository = FilesystemRepository::builder()
    ///     .reader()
    ///     .root("/data/arkad")
    ///     .build();
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
}

// `A: Access` makes `A` `Sync`, so the future of `ensure_root_is_reachable` is `Send`. Without the
// bound, clippy's `future_not_send` lint rejects the method.
impl<A: Access> FilesystemRepository<A> {
    /// Checks that the root exists and is a directory.
    ///
    /// # Errors
    ///
    /// Returns a [`FailedFilesystemOperation`] with:
    /// - [`FailedFilesystemOperationReason::MissingRoot`] if the root does not exist, or a file
    ///   stands in its path.
    /// - [`FailedFilesystemOperationReason::RootIsNotADirectory`] if the root is a file.
    /// - [`FailedFilesystemOperationReason::FailedIo`] if the filesystem cannot inspect the root.
    async fn ensure_root_is_reachable(&self) -> Result<(), FailedFilesystemOperation> {
        match tokio::fs::metadata(&self.root).await {
            Ok(metadata) if metadata.is_dir() => Ok(()),
            Ok(_) => Err(FailedFilesystemOperation::new(
                FailedFilesystemOperationReason::RootIsNotADirectory,
                &self.root,
            )),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) =>
            {
                Err(FailedFilesystemOperation::new(
                    FailedFilesystemOperationReason::MissingRoot,
                    &self.root,
                ))
            }
            Err(error) => Err(FailedFilesystemOperation::from_io(
                FilesystemOperation::Inspect,
                &self.root,
                &error,
            )),
        }
    }
}

impl From<FailedFilesystemOperation> for BackendError {
    /// Converts a [`FailedFilesystemOperation`] into the [`BackendError`] variant for its reason.
    ///
    /// A missing root, a root that is not a directory, and a path that vanished become
    /// [`BackendError::UnreachableStorage`]. A denied permission becomes
    /// [`BackendError::UnauthorizedAccess`]. Every other reason becomes
    /// [`BackendError::FailedOperation`]. The [`FailedFilesystemOperation`] message becomes the
    /// reason.
    fn from(error: FailedFilesystemOperation) -> Self {
        let reason = error.to_string();
        match error.reason {
            FailedFilesystemOperationReason::MissingRoot
            | FailedFilesystemOperationReason::RootIsNotADirectory
            | FailedFilesystemOperationReason::FailedIo {
                kind: io::ErrorKind::NotFound,
                ..
            } => Self::unreachable_storage(reason),
            FailedFilesystemOperationReason::FailedIo {
                kind: io::ErrorKind::PermissionDenied,
                ..
            } => Self::unauthorized_access(reason),
            FailedFilesystemOperationReason::MissingMetadataFile
            | FailedFilesystemOperationReason::InvalidMetadata { .. }
            | FailedFilesystemOperationReason::UnserializableMetadata
            | FailedFilesystemOperationReason::FailedIo { .. } => Self::failed_operation(reason),
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

    /// Builds a repository with read and write access under the given root.
    fn repository_at(root: impl Into<PathBuf>) -> FilesystemRepository<ReadWrite> {
        FilesystemRepository::builder()
            .read_write()
            .root(root)
            .build()
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filesystem_repository() {
        implements_auto_traits::<FilesystemRepository<ReadWrite>>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_filesystem_repository() {
        implements_send::<FilesystemRepository<ReadWrite>>();
    }

    #[test]
    const fn should_implement_sync_for_filesystem_repository() {
        implements_sync::<FilesystemRepository<ReadWrite>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_filesystem_repository() {
        implements_debug::<FilesystemRepository<ReadWrite>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_filesystem_repository() {
        implements_clone::<FilesystemRepository<ReadWrite>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_filesystem_repository() {
        implements_hash::<FilesystemRepository<ReadWrite>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_filesystem_repository() {
        implements_eq::<FilesystemRepository<ReadWrite>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_filesystem_repository() {
        implements_ord::<FilesystemRepository<ReadWrite>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_filesystem_repository() {
        implements_sized::<FilesystemRepository<ReadWrite>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_filesystem_repository() {
        implements_partial_eq::<FilesystemRepository<ReadWrite>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_filesystem_repository() {
        implements_partial_ord::<FilesystemRepository<ReadWrite>>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_filesystem_repository() {
        implements_unpin::<FilesystemRepository<ReadWrite>>();
    }

    const fn implements_read_write_repository<T: ReadWriteRepository>() {}
    #[test]
    const fn should_implement_read_write_repository_for_filesystem_repository() {
        implements_read_write_repository::<FilesystemRepository<ReadWrite>>();
    }

    #[test]
    fn should_hold_the_root_when_building_a_repository() {
        let root = "/data/arkad";

        let expected_result = Path::new("/data/arkad");

        let repository = repository_at(root);
        let result = repository.root();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_join_the_root_and_the_key_when_building_a_document_path() {
        let repository = repository_at("/data/arkad");
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        let expected_result = Path::new("/data/arkad/sec/companyfacts/CIK0000320193.json");

        let result = repository.document_path(&key);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_append_the_metadata_suffix_for_a_document_key() {
        let repository = repository_at("/data/arkad");
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        let expected_result =
            Path::new("/data/arkad/sec/companyfacts/CIK0000320193.json.meta.json");

        let result = repository.metadata_path(&key);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unreachable_storage_when_converting_a_missing_root() {
        let error = FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::MissingRoot,
            "/data/arkad",
        );

        let expected_result = BackendError::unreachable_storage(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Root directory does not exist', Path: '/data/arkad'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unreachable_storage_when_converting_a_root_that_is_not_a_directory() {
        let error = FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::RootIsNotADirectory,
            "/data/arkad",
        );

        let expected_result = BackendError::unreachable_storage(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Root is not a directory', Path: '/data/arkad'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unreachable_storage_when_converting_a_path_that_vanished() {
        let io_error = io::Error::from(io::ErrorKind::NotFound);
        let error = FailedFilesystemOperation::from_io(
            FilesystemOperation::CreateFile,
            "/data/arkad/x.json",
            &io_error,
        );

        let expected_result = BackendError::unreachable_storage(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Failed to create file, entity not found', Path: '/data/arkad/x.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unauthorized_access_when_converting_a_denied_permission() {
        let io_error = io::Error::from(io::ErrorKind::PermissionDenied);
        let error = FailedFilesystemOperation::from_io(
            FilesystemOperation::Read,
            "/data/arkad/x.json",
            &io_error,
        );

        let expected_result = BackendError::unauthorized_access(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Failed to read, permission denied', Path: '/data/arkad/x.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_operation_when_converting_a_full_disk() {
        let io_error = io::Error::from(io::ErrorKind::StorageFull);
        let error = FailedFilesystemOperation::from_io(
            FilesystemOperation::Write,
            "/data/arkad/x.json",
            &io_error,
        );

        let expected_result = BackendError::failed_operation(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Failed to write, no storage space', Path: '/data/arkad/x.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_operation_when_converting_a_missing_metadata_file() {
        let error = FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::MissingMetadataFile,
            "/data/arkad/x.json.meta.json",
        );

        let expected_result = BackendError::failed_operation(
            "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Metadata file does not exist', Path: '/data/arkad/x.json.meta.json'",
        );

        let result = BackendError::from(error);

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_accept_the_root_when_the_root_is_an_existing_directory() {
        let root = temporary_root();
        let repository = repository_at(root.path());

        let expected_result = Ok(());

        let result = repository.ensure_root_is_reachable().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_missing_root_when_the_root_does_not_exist() {
        let parent = temporary_root();
        let root = parent.path().join("missing");
        let repository = repository_at(&root);

        let expected_result = Err(FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::MissingRoot,
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
        let repository = repository_at(&root);

        let expected_result = Err(FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::MissingRoot,
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
        let repository = repository_at(&root);

        let expected_result = Err(FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::RootIsNotADirectory,
            &root,
        ));

        let result = repository.ensure_root_is_reachable().await;

        assert_eq!(result, expected_result);
    }
}
