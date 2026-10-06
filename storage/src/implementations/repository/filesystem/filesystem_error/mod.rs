//! # Filesystem Error
//!
//! Provides [`FilesystemError`], the error raised when the filesystem cannot complete an
//! operation on a path, and the [`FilesystemErrorReason`] that names the cause.

use std::fmt;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

/// Error representing a failed operation on one path of the filesystem.
///
/// Carries the reason and the path it applies to. It names no storage error type, so the code
/// that uses it decides how it surfaces.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[FilesystemError] Filesystem operation failed, Reason: '{reason}', Path: '{}'",
    .path.display()
)]
pub struct FilesystemError {
    /// The reason the operation failed.
    pub reason: FilesystemErrorReason,
    /// The path the operation failed on.
    pub path: PathBuf,
}

impl FilesystemError {
    /// Creates a new [`FilesystemError`] from its reason and the path it applies to.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::implementations::repository::filesystem::{FilesystemError, FilesystemErrorReason};
    ///
    /// let error = FilesystemError::new(FilesystemErrorReason::MissingRoot, "/data/arkad");
    ///
    /// let expected_result = "[FilesystemError] Filesystem operation failed, \
    ///     Reason: 'Root directory does not exist', Path: '/data/arkad'";
    ///
    /// let result = error.to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn new(reason: FilesystemErrorReason, path: impl Into<PathBuf>) -> Self {
        Self {
            reason,
            path: path.into(),
        }
    }

    /// Creates a new [`FilesystemError`] from an I/O error raised by an operation on a path.
    ///
    /// Keeps the kind of the I/O error and drops its operating-system message, so the same
    /// failure reads the same on every platform.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io;
    ///
    /// use storage::implementations::repository::filesystem::{
    ///     FilesystemError, FilesystemErrorReason, FilesystemOperation,
    /// };
    ///
    /// let io_error = io::Error::from(io::ErrorKind::PermissionDenied);
    ///
    /// let expected_result = FilesystemErrorReason::FailedIo {
    ///     operation: FilesystemOperation::Read,
    ///     kind: io::ErrorKind::PermissionDenied,
    /// };
    ///
    /// let error = FilesystemError::from_io(FilesystemOperation::Read, "/data/arkad/x.json", &io_error);
    /// let result = error.reason;
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn from_io(
        operation: FilesystemOperation,
        path: impl Into<PathBuf>,
        io_error: &io::Error,
    ) -> Self {
        Self::new(
            FilesystemErrorReason::FailedIo {
                operation,
                kind: io_error.kind(),
            },
            path,
        )
    }
}

/// Enum representing the reason why a filesystem operation failed.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FilesystemErrorReason {
    /// The root directory does not exist, or a file stands in its path.
    MissingRoot,
    /// The root exists but is not a directory.
    RootIsNotADirectory,
    /// A document exists but its metadata file does not.
    MissingMetadataFile,
    /// The metadata file holds no valid metadata.
    InvalidMetadata {
        /// Line of the first character the parser rejected.
        line: usize,
        /// Column of the first character the parser rejected.
        column: usize,
    },
    /// The metadata cannot be serialized as JSON.
    UnserializableMetadata,
    /// An I/O operation failed.
    FailedIo {
        /// The operation that failed.
        operation: FilesystemOperation,
        /// The kind of I/O error the operation raised.
        kind: io::ErrorKind,
    },
}

impl fmt::Display for FilesystemErrorReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRoot => write!(f, "Root directory does not exist"),
            Self::RootIsNotADirectory => write!(f, "Root is not a directory"),
            Self::MissingMetadataFile => write!(f, "Metadata file does not exist"),
            Self::InvalidMetadata { line, column } => write!(
                f,
                "Metadata file holds no valid metadata at line {line}, column {column}"
            ),
            Self::UnserializableMetadata => write!(f, "Metadata cannot be serialized as JSON"),
            Self::FailedIo { operation, kind } => write!(f, "Failed to {operation}, {kind}"),
        }
    }
}

/// Enum representing a filesystem operation that can fail.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FilesystemOperation {
    /// Reading the metadata of a path, such as whether it is a directory.
    Inspect,
    /// Reading the content of a file.
    Read,
    /// Creating a directory and its missing parents.
    CreateDirectory,
    /// Creating a new file.
    CreateFile,
    /// Writing bytes to an open file.
    Write,
    /// Syncing the content of an open file to disk.
    Sync,
    /// Renaming a file over another path.
    Replace,
}

impl fmt::Display for FilesystemOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verb = match self {
            Self::Inspect => "inspect",
            Self::Read => "read",
            Self::CreateDirectory => "create directory",
            Self::CreateFile => "create file",
            Self::Write => "write",
            Self::Sync => "sync",
            Self::Replace => "replace",
        };
        write!(f, "{verb}")
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;
    use std::path::Path;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filesystem_error() {
        implements_auto_traits::<FilesystemError>();
    }

    #[test]
    const fn should_implement_auto_traits_for_filesystem_error_reason() {
        implements_auto_traits::<FilesystemErrorReason>();
    }

    #[test]
    const fn should_implement_auto_traits_for_filesystem_operation() {
        implements_auto_traits::<FilesystemOperation>();
    }

    const fn implements_error<T: std::error::Error>() {}
    #[test]
    const fn should_implement_error_for_filesystem_error() {
        implements_error::<FilesystemError>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_filesystem_error() {
        implements_value_traits::<FilesystemError>();
    }

    #[test]
    const fn should_implement_value_traits_for_filesystem_error_reason() {
        implements_value_traits::<FilesystemErrorReason>();
    }

    #[test]
    const fn should_implement_value_traits_for_filesystem_operation() {
        implements_value_traits::<FilesystemOperation>();
    }

    #[test]
    fn should_hold_the_reason_and_the_path_when_building_a_filesystem_error() {
        let reason = FilesystemErrorReason::MissingRoot;
        let path = Path::new("/data/arkad");

        let expected_result = FilesystemError {
            reason: FilesystemErrorReason::MissingRoot,
            path: PathBuf::from("/data/arkad"),
        };

        let result = FilesystemError::new(reason, path);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_operation_and_the_kind_when_building_from_an_io_error() {
        let io_error = io::Error::from(io::ErrorKind::StorageFull);
        let path = Path::new("/data/arkad/x.json");

        let expected_result = FilesystemError {
            reason: FilesystemErrorReason::FailedIo {
                operation: FilesystemOperation::Write,
                kind: io::ErrorKind::StorageFull,
            },
            path: PathBuf::from("/data/arkad/x.json"),
        };

        let result = FilesystemError::from_io(FilesystemOperation::Write, path, &io_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_for_a_filesystem_error() {
        let error = FilesystemError::new(FilesystemErrorReason::MissingRoot, "/data/arkad");

        let expected_result = "[FilesystemError] Filesystem operation failed, \
            Reason: 'Root directory does not exist', Path: '/data/arkad'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_the_root_is_not_a_directory() {
        let reason = FilesystemErrorReason::RootIsNotADirectory;

        let expected_result = "Root is not a directory";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_the_metadata_file_is_missing() {
        let reason = FilesystemErrorReason::MissingMetadataFile;

        let expected_result = "Metadata file does not exist";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_the_metadata_is_invalid() {
        let reason = FilesystemErrorReason::InvalidMetadata { line: 3, column: 7 };

        let expected_result = "Metadata file holds no valid metadata at line 3, column 7";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_the_metadata_is_unserializable() {
        let reason = FilesystemErrorReason::UnserializableMetadata;

        let expected_result = "Metadata cannot be serialized as JSON";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_an_io_operation_fails() {
        let reason = FilesystemErrorReason::FailedIo {
            operation: FilesystemOperation::CreateDirectory,
            kind: io::ErrorKind::PermissionDenied,
        };

        let expected_result = "Failed to create directory, permission denied";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }
}
