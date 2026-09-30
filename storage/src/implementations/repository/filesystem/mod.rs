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

use std::ffi::OsStr;
use std::io;
use std::path::{Component, Path, PathBuf};

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
/// `/data/arkad/sec/CIK0000320193.json`. On Windows, a root of `C:\data\arkad` and the same key
/// name `C:\data\arkad\sec\CIK0000320193.json`. The join uses the separator of the platform the
/// store runs on. [`FilesystemRepository::document_path`] performs that join.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilesystemRepository {
    root: PathBuf,
}

impl FilesystemRepository {
    /// Creates a new [`FilesystemRepository`] rooted at the given directory.
    ///
    /// The constructor opens nothing and holds nothing beyond the path. The check reads the root
    /// once, so it catches a misconfigured path at startup rather than on the first operation. It
    /// does not prove that a write succeeds. A read-only root passes here and fails later.
    ///
    /// A leading `~` component stands for the current user's home directory, so `"~/arkad"` roots
    /// the store at `arkad` inside it. The constructor expands only a bare `~`. A `~user` prefix
    /// stays as written. The constructor reads the root's components with the separator rules of
    /// its platform, so `"~\arkad"` expands only on Windows.
    ///
    /// A root that starts with a bare `~` needs a home directory that is known and absolute.
    /// Without one, the constructor fails at startup rather than guessing. The root must be
    /// absolute after any expansion, so the store does not move with the process's working
    /// directory. What counts as absolute follows the platform. A Windows root needs a path
    /// prefix, such as a drive or a share. `"/data/arkad"` carries no prefix, so it is absolute on
    /// Linux and macOS but not on Windows.
    ///
    /// # Errors
    ///
    /// Returns a [`BackendError`] if the root cannot serve as a store:
    /// - [`BackendError::UnreachableStorage`] if the root cannot be reached. Any one of the
    ///   following is enough. The root is empty, or it is not absolute. The root starts with a
    ///   bare `~` while the home directory is unknown, empty, or not absolute. The root does not
    ///   exist, or it is not a directory. The filesystem holding the root did not answer.
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
        Self::with_home_directory(root.into(), std::env::home_dir())
    }

    /// Creates a new [`FilesystemRepository`], expanding a leading `~` to the given home
    /// directory.
    ///
    /// [`FilesystemRepository::new`] passes the process's home directory. A test passes its own, so
    /// the result does not depend on the machine.
    ///
    /// # Errors
    ///
    /// Returns the same [`BackendError`] values [`FilesystemRepository::new`] lists.
    fn with_home_directory(root: PathBuf, home: Option<PathBuf>) -> Result<Self, BackendError> {
        if root.as_os_str().is_empty() {
            return Err(BackendError::unreachable_storage("The root path is empty"));
        }

        let root = expand_home(root, home)?;

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

/// Replaces a leading `~` component of a root with the given home directory.
///
/// A root that does not start with a bare `~` returns unchanged. The constructor reads the root's
/// components with the separator rules of the platform, so a `\` separates them only on Windows.
///
/// # Errors
///
/// Returns a [`BackendError::UnreachableStorage`] if the root starts with a bare `~` and `home`
/// is `None`, empty, or not absolute. Every reason blames the home directory and names the root,
/// so a caller can tell this failure from a root the caller wrote wrong.
fn expand_home(root: PathBuf, home: Option<PathBuf>) -> Result<PathBuf, BackendError> {
    let mut components = root.components();
    if components.next() != Some(Component::Normal(OsStr::new("~"))) {
        return Ok(root);
    }

    let home = home.ok_or_else(|| {
        BackendError::unreachable_storage(format!(
            "The home directory is unknown, so the constructor cannot expand the leading ~ in the \
             root {}",
            root.display()
        ))
    })?;

    if home.as_os_str().is_empty() {
        return Err(BackendError::unreachable_storage(format!(
            "The home directory is empty, so the constructor cannot expand the leading ~ in the \
             root {}",
            root.display()
        )));
    }

    if !home.is_absolute() {
        return Err(BackendError::unreachable_storage(format!(
            "The home directory {} is not an absolute path, so the constructor cannot expand the \
             leading ~ in the root {}",
            home.display(),
            root.display()
        )));
    }

    let rest = components.as_path();

    if rest.as_os_str().is_empty() {
        Ok(home)
    } else {
        Ok(home.join(rest))
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

    /// An absolute home directory on the platform the tests run on.
    const ABSOLUTE_HOME: &str = if cfg!(windows) {
        r"C:\Users\arkad-user"
    } else {
        "/home/arkad-user"
    };

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
    fn should_open_the_store_in_the_home_directory_when_the_root_is_a_tilde() {
        let home = std::env::temp_dir();
        let expected_result = home.clone();

        let repository = FilesystemRepository::with_home_directory(PathBuf::from("~"), Some(home))
            .expect(
                "Given an existing directory as the home directory, the store should always open",
            );

        assert_eq!(repository.root(), expected_result);
    }

    #[test]
    fn should_expand_the_tilde_from_the_process_home_directory_when_opening_the_store() {
        let expected_result =
            FilesystemRepository::with_home_directory(PathBuf::from("~"), std::env::home_dir());

        let result = FilesystemRepository::new("~");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_open_the_store_when_the_root_is_empty() {
        let expected_result = BackendError::unreachable_storage("The root path is empty");

        let result =
            FilesystemRepository::new("").expect_err("An empty root should never open as a store");

        assert_eq!(result, expected_result);
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
    fn should_fail_to_open_the_store_when_the_home_directory_is_unknown() {
        let expected_result = expand_home(PathBuf::from("~/arkad"), None)
            .expect_err("A tilde root with no home directory should never expand");

        let result = FilesystemRepository::with_home_directory(PathBuf::from("~/arkad"), None)
            .expect_err("A tilde root with no home directory should never open as a store");

        assert_eq!(result, expected_result);
    }

    #[test]
    #[cfg_attr(
        not(windows),
        ignore = "A rooted path needs a Windows prefix only on Windows"
    )]
    fn should_fail_to_open_the_store_when_the_root_is_relative_because_it_carries_no_prefix() {
        let expected_result =
            BackendError::unreachable_storage("/data/arkad is not an absolute path");

        let result = FilesystemRepository::new("/data/arkad")
            .expect_err("A root with no Windows prefix should never open as a store");

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
    fn should_join_the_root_and_the_key_when_building_a_document_path() {
        let repository = FilesystemRepository::new(std::env::temp_dir()).expect(
            "Given the system temporary directory, opening the store should always succeed",
        );
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");
        let expected_result = std::env::temp_dir()
            .join("sec")
            .join("companyfacts")
            .join("CIK0000320193.json");

        let result = repository.document_path(&key);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_join_the_home_directory_and_the_rest_when_the_root_starts_with_a_tilde() {
        let home = Path::new(ABSOLUTE_HOME);
        let expected_result = home.join("Projects").join("arkad");

        let result = expand_home(PathBuf::from("~/Projects/arkad"), Some(home.to_path_buf()))
            .expect("Given an absolute home directory, the expansion should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_home_directory_when_the_root_is_a_bare_tilde() {
        let expected_result = PathBuf::from(ABSOLUTE_HOME);

        let result = expand_home(PathBuf::from("~"), Some(expected_result.clone()))
            .expect("Given an absolute home directory, the expansion should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_root_unchanged_when_it_names_another_user() {
        let expected_result = PathBuf::from("~other/arkad");

        let result = expand_home(expected_result.clone(), None).expect(
            "Given a root whose tilde names another user, the expansion should succeed without a \
             home directory",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_root_unchanged_when_it_starts_with_no_tilde() {
        let expected_result = PathBuf::from("/data/arkad");

        let result = expand_home(expected_result.clone(), None).expect(
            "Given a root with no tilde, the expansion should succeed without a home directory",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    #[cfg_attr(windows, ignore = "A backslash separates components only on Windows")]
    fn should_keep_the_root_unchanged_when_a_backslash_follows_the_tilde() {
        let expected_result = PathBuf::from(r"~\arkad");

        let result = expand_home(expected_result.clone(), None).expect(
            "Outside Windows a backslash is an ordinary character, so the root should stay \
             unchanged without a home directory",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_expand_the_root_when_the_home_directory_is_unknown() {
        let expected_result = BackendError::unreachable_storage(
            "The home directory is unknown, so the constructor cannot expand the leading ~ in the \
             root ~/arkad",
        );

        let result = expand_home(PathBuf::from("~/arkad"), None)
            .expect_err("A tilde root with no home directory should never expand");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_expand_the_root_when_the_home_directory_is_empty() {
        let expected_result = BackendError::unreachable_storage(
            "The home directory is empty, so the constructor cannot expand the leading ~ in the \
             root ~/arkad",
        );

        let result = expand_home(PathBuf::from("~/arkad"), Some(PathBuf::new()))
            .expect_err("A tilde root with an empty home directory should never expand");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_expand_the_root_when_the_home_directory_is_relative() {
        let expected_result = BackendError::unreachable_storage(
            "The home directory arkad-home is not an absolute path, so the constructor cannot \
             expand the leading ~ in the root ~/arkad",
        );

        let result = expand_home(PathBuf::from("~/arkad"), Some(PathBuf::from("arkad-home")))
            .expect_err("A tilde root with a relative home directory should never expand");

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
