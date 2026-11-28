//! # Unit Write Error
//!
//! Provides [`UnitWriteError`], the error of the [`UnitWriter`](crate::datasets::UnitWriter) and
//! of [`write_task_manifest`](crate::datasets::write_task_manifest()) when they cannot write a
//! unit or the list file of a task.

use storage::WriteError;
use thiserror::Error;
use uuid::Uuid;

use crate::datasets::{ArrowRowError, PartWriteError, TaskAttempt, UnitReadError};
use crate::shared::row_digest::DuplicateDigestGroup;

/// Error of a unit, or of the list file of a task, that the writer did not write.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnitWriteError {
    /// The writer did not read the current unit of the key.
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    FailedCurrentUnitRead(#[from] UnitReadError),

    /// The writer did not write a part of the unit.
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    FailedPartWrite(#[from] PartWriteError),

    /// The rows of one digest group came in two calls, so the digest would depend on the parts.
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    SplitDigestGroup(#[from] DuplicateDigestGroup),

    /// The rows of the list file do not turn into a record batch.
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    FailedManifestConversion(#[from] ArrowRowError),

    /// A unit was written by another execution or attempt than the task of the list file.
    #[error(
        "[InvalidTaskLocation] Unit '{unit_key}' was written by execution '{execution_id}', attempt '{attempt}', not by the task of the list file"
    )]
    InvalidTaskLocation {
        /// The key of the unit, for example `2024q1`.
        unit_key: String,
        /// The execution that wrote the parts of the unit.
        execution_id: Uuid,
        /// The attempt that wrote the parts of the unit.
        attempt: TaskAttempt,
    },

    /// Parquet rejects the batch or the properties of the list file.
    #[error(
        "[FailedManifestEncoding] Failed to encode the unit manifests as Parquet, Reason: '{reason}'"
    )]
    FailedManifestEncoding {
        /// The text of the Parquet error.
        reason: String,
    },

    /// The store did not write the list file, for example because a blob exists at its key.
    #[error("[FailedManifestWrite] Failed to write the unit manifests, Caused by: {source}")]
    FailedManifestWrite {
        /// The error of the store.
        #[source]
        source: WriteError,
    },
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_unit_write_error() {
        implements_auto_traits::<UnitWriteError>();
    }

    #[test]
    fn should_print_only_the_inner_error_when_the_part_write_fails() {
        let error = UnitWriteError::from(PartWriteError::InvalidUnitKey {
            unit_key: "2024/q1".to_owned(),
        });

        let expected_result = "[InvalidUnitKey] Unit key '2024/q1' is not one component of a key";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_unit_and_its_attempt_when_the_task_location_differs() {
        let error = UnitWriteError::InvalidTaskLocation {
            unit_key: "2024q1".to_owned(),
            execution_id: Uuid::nil(),
            attempt: TaskAttempt {
                number: 1,
                is_bucket_split: true,
            },
        };

        let expected_result = "[InvalidTaskLocation] Unit '2024q1' was written by execution '00000000-0000-0000-0000-000000000000', attempt '1-b', not by the task of the list file";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_reason_when_the_manifest_encoding_fails() {
        let error = UnitWriteError::FailedManifestEncoding {
            reason: "Invalid zstd compression level".to_owned(),
        };

        let expected_result = "[FailedManifestEncoding] Failed to encode the unit manifests as Parquet, Reason: 'Invalid zstd compression level'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_the_store_error_when_the_manifest_write_fails() {
        let error = UnitWriteError::FailedManifestWrite {
            source: WriteError::conflicting_write("Blob 'unit_manifests/a.parquet' exists"),
        };

        let expected_result = "[FailedManifestWrite] Failed to write the unit manifests, Caused by: [ConflictingWrite] Write conflicts with existing data, Reason: 'Blob 'unit_manifests/a.parquet' exists'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
