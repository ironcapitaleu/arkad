//! # Failed Filesystem Operation
//!
//! Provides [`FailedFilesystemOperation`], the error raised when the filesystem cannot complete an
//! operation on a path, and the [`FailedFilesystemOperationReason`] that names the cause.

use std::fmt;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::implementations::repository::filesystem::METADATA_SUFFIX;

/// Error representing a failed operation on one path of the filesystem.
///
/// Carries the reason and the path it applies to.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[FailedFilesystemOperation] Filesystem operation failed, Reason: '{reason}', Path: '{}'",
    .path.display()
)]
pub struct FailedFilesystemOperation {
    /// The reason the operation failed.
    pub reason: FailedFilesystemOperationReason,
    /// The path the operation failed on.
    pub path: PathBuf,
}

impl FailedFilesystemOperation {
    /// Creates a new [`FailedFilesystemOperation`] from its reason and the path it applies to.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::implementations::repository::filesystem::{
    ///     FailedFilesystemOperation, FailedFilesystemOperationReason,
    /// };
    ///
    /// let error = FailedFilesystemOperation::new(
    ///     FailedFilesystemOperationReason::MissingRoot,
    ///     "/data/arkad",
    /// );
    ///
    /// let expected_result = "[FailedFilesystemOperation] Filesystem operation failed, \
    ///     Reason: 'Root directory does not exist', Path: '/data/arkad'";
    ///
    /// let result = error.to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn new(reason: FailedFilesystemOperationReason, path: impl Into<PathBuf>) -> Self {
        Self {
            reason,
            path: path.into(),
        }
    }

    /// Creates a new [`FailedFilesystemOperation`] from the I/O error of an operation on a path.
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
    ///     FailedFilesystemOperation, FailedFilesystemOperationReason, FilesystemOperation,
    /// };
    ///
    /// let io_error = io::Error::from(io::ErrorKind::PermissionDenied);
    ///
    /// let expected_result = FailedFilesystemOperationReason::FailedIo {
    ///     operation: FilesystemOperation::Read,
    ///     kind: io::ErrorKind::PermissionDenied,
    /// };
    ///
    /// let error = FailedFilesystemOperation::from_io(
    ///     FilesystemOperation::Read,
    ///     "/data/arkad/x.json",
    ///     &io_error,
    /// );
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
            FailedFilesystemOperationReason::FailedIo {
                operation,
                kind: io_error.kind(),
            },
            path,
        )
    }
}

/// Enum representing the reason why a filesystem operation failed.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FailedFilesystemOperationReason {
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
    /// The metadata holds another document kind or kind version than the repository's document
    /// type.
    UnexpectedMetadataKind {
        /// The kind the metadata holds.
        found_kind: String,
        /// The kind version the metadata holds.
        found_kind_version: u32,
        /// The kind of the repository's document type.
        expected_kind: String,
        /// The kind version of the repository's document type.
        expected_kind_version: u32,
    },
    /// The metadata cannot be serialized as JSON.
    UnserializableMetadata,
    /// The document size differs from the size its metadata names.
    UnexpectedDocumentSize {
        /// The number of bytes the document holds.
        found_size: u64,
        /// The number of bytes the metadata names.
        expected_size: u64,
    },
    /// The document digest differs from the SHA-256 digest its metadata names.
    UnexpectedDocumentDigest {
        /// The digest of the bytes the document holds.
        found_digest: String,
        /// The digest the metadata names.
        expected_digest: String,
    },
    /// The key ends in the suffix of a metadata file, so it names the metadata file of another key.
    InvalidKeySuffix,
    /// An I/O operation failed.
    FailedIo {
        /// The operation that failed.
        operation: FilesystemOperation,
        /// The kind of I/O error the operation raised.
        kind: io::ErrorKind,
    },
}

impl fmt::Display for FailedFilesystemOperationReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRoot => write!(f, "Root directory does not exist"),
            Self::RootIsNotADirectory => write!(f, "Root is not a directory"),
            Self::MissingMetadataFile => write!(f, "Metadata file does not exist"),
            Self::InvalidMetadata { line, column } => write!(
                f,
                "Metadata file holds no valid metadata at line {line}, column {column}"
            ),
            Self::UnexpectedMetadataKind {
                found_kind,
                found_kind_version,
                expected_kind,
                expected_kind_version,
            } => write!(
                f,
                "Metadata holds kind '{found_kind}' version {found_kind_version}, expected kind \
                 '{expected_kind}' version {expected_kind_version}"
            ),
            Self::UnserializableMetadata => write!(f, "Metadata cannot be serialized as JSON"),
            Self::UnexpectedDocumentSize {
                found_size,
                expected_size,
            } => write!(
                f,
                "Document holds {found_size} bytes, metadata names {expected_size} bytes"
            ),
            Self::UnexpectedDocumentDigest {
                found_digest,
                expected_digest,
            } => write!(
                f,
                "Document has SHA-256 '{found_digest}', metadata names '{expected_digest}'"
            ),
            Self::InvalidKeySuffix => write!(
                f,
                "Key ends in '{METADATA_SUFFIX}', the suffix of a metadata file"
            ),
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
    const fn should_implement_auto_traits_for_failed_filesystem_operation() {
        implements_auto_traits::<FailedFilesystemOperation>();
    }

    #[test]
    const fn should_implement_auto_traits_for_failed_filesystem_operation_reason() {
        implements_auto_traits::<FailedFilesystemOperationReason>();
    }

    #[test]
    const fn should_implement_auto_traits_for_filesystem_operation() {
        implements_auto_traits::<FilesystemOperation>();
    }

    const fn implements_error<T: std::error::Error>() {}
    #[test]
    const fn should_implement_error_for_failed_filesystem_operation() {
        implements_error::<FailedFilesystemOperation>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_failed_filesystem_operation() {
        implements_value_traits::<FailedFilesystemOperation>();
    }

    #[test]
    const fn should_implement_value_traits_for_failed_filesystem_operation_reason() {
        implements_value_traits::<FailedFilesystemOperationReason>();
    }

    #[test]
    const fn should_implement_value_traits_for_filesystem_operation() {
        implements_value_traits::<FilesystemOperation>();
    }

    #[test]
    fn should_hold_the_reason_and_the_path_when_building_a_failed_filesystem_operation() {
        let reason = FailedFilesystemOperationReason::MissingRoot;
        let path = Path::new("/data/arkad");

        let expected_result = FailedFilesystemOperation {
            reason: FailedFilesystemOperationReason::MissingRoot,
            path: PathBuf::from("/data/arkad"),
        };

        let result = FailedFilesystemOperation::new(reason, path);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_operation_and_the_kind_when_building_from_an_io_error() {
        let io_error = io::Error::from(io::ErrorKind::StorageFull);
        let path = Path::new("/data/arkad/x.json");

        let expected_result = FailedFilesystemOperation {
            reason: FailedFilesystemOperationReason::FailedIo {
                operation: FilesystemOperation::Write,
                kind: io::ErrorKind::StorageFull,
            },
            path: PathBuf::from("/data/arkad/x.json"),
        };

        let result =
            FailedFilesystemOperation::from_io(FilesystemOperation::Write, path, &io_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_for_a_failed_filesystem_operation() {
        let error = FailedFilesystemOperation::new(
            FailedFilesystemOperationReason::MissingRoot,
            "/data/arkad",
        );

        let expected_result = "[FailedFilesystemOperation] Filesystem operation failed, \
            Reason: 'Root directory does not exist', Path: '/data/arkad'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_the_root_is_not_a_directory() {
        let reason = FailedFilesystemOperationReason::RootIsNotADirectory;

        let expected_result = "Root is not a directory";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_the_metadata_file_is_missing() {
        let reason = FailedFilesystemOperationReason::MissingMetadataFile;

        let expected_result = "Metadata file does not exist";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_the_metadata_is_invalid() {
        let reason = FailedFilesystemOperationReason::InvalidMetadata { line: 3, column: 7 };

        let expected_result = "Metadata file holds no valid metadata at line 3, column 7";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_the_metadata_is_unserializable() {
        let reason = FailedFilesystemOperationReason::UnserializableMetadata;

        let expected_result = "Metadata cannot be serialized as JSON";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_an_io_operation_fails() {
        let reason = FailedFilesystemOperationReason::FailedIo {
            operation: FilesystemOperation::CreateDirectory,
            kind: io::ErrorKind::PermissionDenied,
        };

        let expected_result = "Failed to create directory, permission denied";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_the_metadata_kind_is_unexpected() {
        let reason = FailedFilesystemOperationReason::UnexpectedMetadataKind {
            found_kind: "other_document".to_owned(),
            found_kind_version: 2,
            expected_kind: "sample_document".to_owned(),
            expected_kind_version: 1,
        };

        let expected_result = "Metadata holds kind 'other_document' version 2, expected kind \
                               'sample_document' version 1";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_the_document_size_is_unexpected() {
        let reason = FailedFilesystemOperationReason::UnexpectedDocumentSize {
            found_size: 3,
            expected_size: 2,
        };

        let expected_result = "Document holds 3 bytes, metadata names 2 bytes";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_the_document_digest_is_unexpected() {
        let reason = FailedFilesystemOperationReason::UnexpectedDocumentDigest {
            found_digest: "ab".to_owned(),
            expected_digest: "cd".to_owned(),
        };

        let expected_result = "Document has SHA-256 'ab', metadata names 'cd'";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_as_expected_when_the_key_suffix_is_invalid() {
        let reason = FailedFilesystemOperationReason::InvalidKeySuffix;

        let expected_result = "Key ends in '.meta.json', the suffix of a metadata file";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }
}
