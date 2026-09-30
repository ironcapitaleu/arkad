//! # Filesystem Repository
//!
//! Provides [`FilesystemRepository`], the adapter that stores documents as files under a root
//! directory.
//!
//! ## Record and Key
//!
//! [`ReadRepository`](crate::ReadRepository) and [`WriteRepository`](crate::WriteRepository) each
//! leave their associated types to the implementor. This adapter pins `Record` to [`RawDocument`]
//! and `Key` to [`DocumentKey`].
//!
//! ## Modules
//!
//! - [`document_key`]: The [`DocumentKey`] naming one document, and the error for a rejected path.
//! - [`document_metadata`]: The [`DocumentMetadata`] stored beside a document.
//! - [`raw_document`]: The [`RawDocument`] a read returns and a write accepts.
//!
//! ## Usage
//!
//! ```rust
//! use storage::implementations::repository::filesystem::FilesystemRepository;
//!
//! let repository = FilesystemRepository::new(std::env::temp_dir())
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
/// The root plays the part a table plays in a database, and a [`DocumentKey`] is the path under it.
/// A document lives at the root joined with its key, so a root of `/data/arkad` and a key of
/// `sec/CIK0000320193.json` name `/data/arkad/sec/CIK0000320193.json`. The join uses the
/// separator of the platform the store runs on.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilesystemRepository {
    root: PathBuf,
}

impl FilesystemRepository {
    /// Creates a new [`FilesystemRepository`] rooted at the given absolute directory.
    ///
    /// The constructor opens nothing and holds nothing beyond the path. The check reads the root
    /// once, so it catches a misconfigured path at startup rather than on the first operation. It
    /// does not prove that a write succeeds. A read-only root passes here and fails later.
    ///
    /// The root must be absolute, so the store does not move with the process's working directory.
    /// The constructor expands nothing, so a `~` is a directory named `~`, not the home directory.
    ///
    /// # Errors
    ///
    /// Returns a [`BackendError`] if the root cannot serve as a store:
    /// - [`BackendError::UnreachableStorage`] if the root is not absolute, does not exist, or
    ///   exists but is not a directory.
    /// - [`BackendError::UnauthorizedAccess`] if the process cannot read the root.
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
}

/// Maps a filesystem failure onto the storage crate's backend error.
///
/// The mapping follows what the caller can do about the failure. An absent or unreachable root is
/// a storage problem. A refused request is a permission problem. Anything else is a failed
/// operation.
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

    use crate::traits::repository::ReadWriteRepository;
    use pretty_assertions::assert_eq;

    use super::*;

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
        let expected_result = std::env::temp_dir();

        let repository = FilesystemRepository::new(std::env::temp_dir()).expect(
            "Given the system temporary directory, opening the store should always succeed",
        );

        assert_eq!(repository.root(), expected_result);
    }

    #[test]
    fn should_fail_to_open_the_store_when_the_root_is_relative() {
        let expected_result =
            BackendError::unreachable_storage("arkad/data is not an absolute path");

        let result = FilesystemRepository::new("arkad/data")
            .expect_err("A relative root should never open as a store");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_open_the_store_when_the_root_does_not_exist() {
        let missing = std::env::temp_dir().join(format!(
            "arkad-storage-filesystem-absent-root-{}",
            std::process::id()
        ));

        let result = FilesystemRepository::new(missing)
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
