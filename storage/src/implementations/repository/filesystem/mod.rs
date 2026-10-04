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
//! ## Modules
//!
//! - [`document_key`]: The [`DocumentKey`] naming one document, and the [`InvalidDocumentKey`] for
//!   a rejected path.
//! - [`document_metadata`]: The [`DocumentMetadata`] describing the fetch behind a document.
//! - [`raw_document`]: The [`RawDocument`] a read returns and a write accepts.

use std::io;
use std::path::{Path, PathBuf};

use crate::error::BackendError;

pub mod document_key;
pub mod document_metadata;
pub mod raw_document;
mod read_repository;
mod write_repository;

pub use document_key::{DocumentKey, InvalidDocumentKey};
pub use document_metadata::DocumentMetadata;
pub use raw_document::RawDocument;

/// Stores documents as files beneath one root directory.
///
/// A [`DocumentKey`] is a document's path under the root. A document lives at the root joined with
/// its key. On Linux and macOS, a root of `/data/arkad` and a key of `sec/CIK0000320193.json` name
/// `/data/arkad/sec/CIK0000320193.json`. On Windows, the same key under a root of `C:\data\arkad`
/// names `C:\data\arkad\sec\CIK0000320193.json`. The join uses the separator of the platform the
/// store runs on. [`FilesystemRepository::document_path`] performs that join.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilesystemRepository {
    root: PathBuf,
}

impl FilesystemRepository {
    /// Creates a new [`FilesystemRepository`] rooted at the given directory.
    ///
    /// The constructor opens nothing and holds nothing beyond the path. It looks the root up once,
    /// so a misconfigured path fails at startup rather than on the first operation. It checks only
    /// that the root exists and is a directory. It checks no read or write access, because
    /// permissions can differ for each document under the root. Each operation reports a denied
    /// permission for the document it touches.
    ///
    /// The root must be absolute, so the store does not move with the process's working directory.
    /// What counts as absolute follows the platform. A Windows root needs a path prefix, such as a
    /// drive or a share. `"/data/arkad"` carries no prefix, so it is absolute on Linux and macOS
    /// but not on Windows.
    ///
    /// # Errors
    ///
    /// Returns a [`BackendError`] if the root cannot serve as a store:
    /// - [`BackendError::UnreachableStorage`] if the root cannot be reached. Any one of the
    ///   following is enough. The root is empty, or it is not absolute. The root does not exist, or
    ///   it is not a directory. The filesystem holding the root did not answer.
    /// - [`BackendError::UnauthorizedAccess`] if a directory above the root denies the lookup.
    /// - [`BackendError::FailedOperation`] for any other failure while inspecting the root.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::implementations::repository::filesystem::FilesystemRepository;
    ///
    /// let repository = FilesystemRepository::new(std::env::temp_dir())
    ///     .expect("Given the system temporary directory, the root check should always succeed");
    ///
    /// let expected_result = true;
    ///
    /// let result = repository.root().is_dir();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, BackendError> {
        let root = root.into();

        if root.as_os_str().is_empty() {
            return Err(BackendError::unreachable_storage("The root path is empty"));
        }

        if !root.is_absolute() {
            return Err(BackendError::unreachable_storage(format!(
                "{} is not an absolute path",
                root.display()
            )));
        }

        let metadata = std::fs::metadata(&root).map_err(|error| to_backend_error(&error))?;

        if metadata.is_dir() {
            Ok(Self { root })
        } else {
            Err(BackendError::unreachable_storage(format!(
                "{} is not a directory",
                root.display()
            )))
        }
    }

    /// Returns the absolute root directory every key resolves under.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the path on disk of the document a key names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::implementations::repository::filesystem::{DocumentKey, FilesystemRepository};
    ///
    /// let repository = FilesystemRepository::new(std::env::temp_dir())
    ///     .expect("Given the system temporary directory, the root check should always succeed");
    /// let key = DocumentKey::new("sec/CIK0000320193.json")
    ///     .expect("Given a valid relative path, the key should always build");
    ///
    /// let expected_result = std::env::temp_dir().join("sec").join("CIK0000320193.json");
    ///
    /// let result = repository.document_path(&key);
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn document_path(&self, key: &DocumentKey) -> PathBuf {
        self.root.join(key.as_path())
    }
}

/// Maps a filesystem failure onto the storage crate's backend error.
///
/// The mapping follows what the caller can do about the failure. A path that is absent, or that
/// the filesystem cannot reach, is a storage problem. A denied permission is a permission problem.
/// Anything else is a failed operation.
///
/// A caller that treats an absent file as a result rather than a failure must check for
/// [`io::ErrorKind::NotFound`] before it calls this function.
fn to_backend_error(error: &io::Error) -> BackendError {
    let reason = error.to_string();

    match error.kind() {
        io::ErrorKind::PermissionDenied => BackendError::unauthorized_access(reason),
        io::ErrorKind::NotFound
        | io::ErrorKind::NotConnected
        | io::ErrorKind::ConnectionRefused
        | io::ErrorKind::ConnectionAborted
        | io::ErrorKind::TimedOut => BackendError::unreachable_storage(reason),
        _ => BackendError::failed_operation(reason),
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::traits::repository::ReadWriteRepository;

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
    fn should_open_the_store_when_the_root_is_an_existing_directory() {
        let root = std::env::temp_dir();

        let expected_result = std::env::temp_dir();

        let repository = FilesystemRepository::new(root).expect(
            "Given the system temporary directory, opening the store should always succeed",
        );
        let result = repository.root();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_open_the_store_when_the_root_is_empty() {
        let root = "";

        let expected_result = BackendError::unreachable_storage("The root path is empty");

        let result = FilesystemRepository::new(root)
            .expect_err("An empty root should never open as a store");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_open_the_store_when_the_root_is_relative() {
        let root = "arkad/data";

        let expected_result =
            BackendError::unreachable_storage("arkad/data is not an absolute path");

        let result = FilesystemRepository::new(root)
            .expect_err("A relative root should never open as a store");

        assert_eq!(result, expected_result);
    }

    #[test]
    #[cfg_attr(
        not(windows),
        ignore = "A rooted path needs a Windows prefix only on Windows"
    )]
    fn should_fail_to_open_the_store_when_the_root_is_relative_because_it_carries_no_prefix() {
        let root = "/data/arkad";

        let expected_result =
            BackendError::unreachable_storage("/data/arkad is not an absolute path");

        let result = FilesystemRepository::new(root)
            .expect_err("A root with no Windows prefix should never open as a store");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_open_the_store_when_the_root_does_not_exist() {
        let missing_root = std::env::temp_dir().join(format!(
            "arkad-storage-filesystem-absent-root-{}",
            std::process::id()
        ));

        let result = FilesystemRepository::new(missing_root)
            .expect_err("A root that does not exist should never open as a store");

        assert!(matches!(result, BackendError::UnreachableStorage { .. }));
    }

    #[test]
    fn should_fail_to_open_the_store_when_the_root_is_a_file() {
        let root = std::env::temp_dir().join(format!(
            "arkad-storage-filesystem-root-is-a-file-{}",
            std::process::id()
        ));
        std::fs::write(&root, b"").expect(
            "Given the system temporary directory, writing an empty file should always succeed",
        );

        let expected_result =
            BackendError::unreachable_storage(format!("{} is not a directory", root.display()));

        let outcome = FilesystemRepository::new(&root);
        std::fs::remove_file(&root)
            .expect("Given a file this test just created, removing it should always succeed");
        let result = outcome.expect_err("A root that is a file should never open as a store");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_join_the_root_and_the_key_when_building_a_document_path() {
        let root = std::env::temp_dir();
        let repository = FilesystemRepository::new(&root).expect(
            "Given the system temporary directory, opening the store should always succeed",
        );
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        // "<system temporary directory>/sec/companyfacts/CIK0000320193.json"
        let expected_result = root
            .join("sec")
            .join("companyfacts")
            .join("CIK0000320193.json");

        let result = repository.document_path(&key);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_map_a_permission_denied_failure_to_unauthorized_access() {
        let error = io::Error::from(io::ErrorKind::PermissionDenied);

        let expected_result = BackendError::unauthorized_access(error.to_string());

        let result = to_backend_error(&error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_map_a_not_found_failure_to_unreachable_storage() {
        let error = io::Error::from(io::ErrorKind::NotFound);

        let expected_result = BackendError::unreachable_storage(error.to_string());

        let result = to_backend_error(&error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_map_a_connection_refused_failure_to_unreachable_storage() {
        let error = io::Error::from(io::ErrorKind::ConnectionRefused);

        let expected_result = BackendError::unreachable_storage(error.to_string());

        let result = to_backend_error(&error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_map_a_timed_out_failure_to_unreachable_storage() {
        let error = io::Error::from(io::ErrorKind::TimedOut);

        let expected_result = BackendError::unreachable_storage(error.to_string());

        let result = to_backend_error(&error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_map_any_other_failure_to_failed_operation() {
        let error = io::Error::from(io::ErrorKind::OutOfMemory);

        let expected_result = BackendError::failed_operation(error.to_string());

        let result = to_backend_error(&error);

        assert_eq!(result, expected_result);
    }
}
