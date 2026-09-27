//! # File System Repository
//!
//! Provides [`FileSystemRepository`], the adapter that stores documents as files under a root
//! directory.
//!
//! ## Record and Key
//!
//! [`ReadRepository`] and [`WriteRepository`] each leave their associated types to the implementor.
//! This adapter pins `Record` to [`RawDocument`] and `Key` to [`DocumentKey`]. Both types belong to
//! this crate, because the Transform stage has no settled output type yet. A Load sub-state
//! replaces them once it does.
//!
//! ## Usage
//!
//! ```rust
//! use storage_filesystem::FileSystemRepository;
//!
//! let repository = FileSystemRepository::new(std::env::temp_dir())
//!     .expect("Given the system temporary directory, the root check should always succeed");
//!
//! let expected_result = std::env::temp_dir();
//!
//! let result = repository.root();
//!
//! assert_eq!(result, expected_result);
//! ```

use std::io;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use storage::{BackendError, ReadError, ReadRepository, WriteError, WriteRepository};

use crate::document_key::DocumentKey;
use crate::raw_document::RawDocument;

/// Stores documents as files beneath one root directory.
///
/// The root plays the part a table plays in a database, and a [`DocumentKey`] is the path under it.
/// A document and its metadata are two files, so the metadata is written first and the document's
/// existence marks the write as committed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileSystemRepository {
    root: PathBuf,
}

impl FileSystemRepository {
    /// Checks that the given directory can serve as the store's root, and holds it.
    ///
    /// Nothing is opened and nothing is held beyond the path. The check reads the root once, so it
    /// catches a misconfigured path at startup rather than on the first operation. It does not
    /// prove that a write succeeds: a read-only root passes here and fails later.
    ///
    /// # Errors
    ///
    /// Returns a [`BackendError`] if the root cannot serve as a store:
    /// - [`BackendError::UnreachableStorage`] if the root does not exist, or exists but is not a
    ///   directory.
    /// - [`BackendError::UnauthorizedAccess`] if the process cannot read the root.
    /// - [`BackendError::FailedOperation`] for any other failure while inspecting the root.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage_filesystem::FileSystemRepository;
    ///
    /// let repository = FileSystemRepository::new(std::env::temp_dir())
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
        let metadata = std::fs::metadata(&root).map_err(|error| to_backend_error(&error))?;

        if metadata.is_dir() {
            Ok(Self { root })
        } else {
            Err(BackendError::unreachable_storage(format!(
                "'{}' is not a directory",
                root.display()
            )))
        }
    }

    /// Returns the root directory every key resolves under.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// Maps a filesystem failure onto the storage crate's backend error.
///
/// The mapping follows what the caller can do about the failure. A root that is absent or
/// unreachable is a storage problem, a refused request is a permission problem, and anything else
/// is a failed operation.
///
/// A missing document is not a backend failure. [`ReadRepository::get`] reports that as `None`.
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

#[async_trait]
impl ReadRepository for FileSystemRepository {
    type Record = RawDocument;
    type Key = DocumentKey;

    /// Reads the document a key names, together with its metadata.
    ///
    /// # Errors
    ///
    /// Returns a [`ReadError::Backend`] wrapping the [`BackendError`] the filesystem produced.
    async fn get(&self, _key: Self::Key) -> Result<Option<Self::Record>, ReadError> {
        todo!("reads against the filesystem arrive with the follow-up ticket")
    }
}

#[async_trait]
impl WriteRepository for FileSystemRepository {
    type Record = RawDocument;

    /// Writes a document and its metadata under the root.
    ///
    /// # Errors
    ///
    /// Returns a [`WriteError::Backend`] wrapping the [`BackendError`] the filesystem produced.
    async fn persist(&self, _record: Self::Record) -> Result<(), WriteError> {
        todo!("writes against the filesystem arrive with the follow-up ticket")
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;
    use storage::ReadWriteRepository;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_when_using_file_system_repository() {
        implements_auto_traits::<FileSystemRepository>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_using_file_system_repository() {
        implements_send::<FileSystemRepository>();
    }

    #[test]
    const fn should_implement_sync_when_using_file_system_repository() {
        implements_sync::<FileSystemRepository>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_file_system_repository() {
        implements_debug::<FileSystemRepository>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_when_using_file_system_repository() {
        implements_clone::<FileSystemRepository>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_when_using_file_system_repository() {
        implements_hash::<FileSystemRepository>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_when_using_file_system_repository() {
        implements_eq::<FileSystemRepository>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_when_using_file_system_repository() {
        implements_ord::<FileSystemRepository>();
    }

    const fn implements_read_repository<T: ReadRepository>() {}
    #[test]
    const fn should_implement_read_repository_for_file_system_repository() {
        implements_read_repository::<FileSystemRepository>();
    }

    const fn implements_write_repository<T: WriteRepository>() {}
    #[test]
    const fn should_implement_write_repository_for_file_system_repository() {
        implements_write_repository::<FileSystemRepository>();
    }

    const fn implements_read_write_repository<T: ReadWriteRepository>() {}
    #[test]
    const fn should_implement_read_write_repository_for_file_system_repository() {
        implements_read_write_repository::<FileSystemRepository>();
    }

    #[test]
    fn should_open_the_store_when_the_root_is_an_existing_directory() {
        let expected_result = std::env::temp_dir();

        let repository = FileSystemRepository::new(std::env::temp_dir()).expect(
            "Given the system temporary directory, opening the store should always succeed",
        );

        assert_eq!(repository.root(), expected_result);
    }

    #[test]
    fn should_fail_to_open_the_store_when_the_root_does_not_exist() {
        let missing = std::env::temp_dir().join("arkad-storage-filesystem-absent-root");

        let result = FileSystemRepository::new(missing)
            .expect_err("A root that does not exist should never open as a store");

        assert!(matches!(result, BackendError::UnreachableStorage { .. }));
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
