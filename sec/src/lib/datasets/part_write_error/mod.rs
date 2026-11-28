//! # Part Write Error
//!
//! Provides [`PartWriteError`], the error of [`write_part`](crate::datasets::write_part()) when it
//! cannot encode or store one part of a unit.

use storage::WriteError;
use thiserror::Error;

use crate::datasets::ArrowRowError;

/// Error of a part that the writer did not write.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PartWriteError {
    /// The unit key is empty, holds a `/`, or holds a character that no key can hold.
    #[error("[InvalidUnitKey] Unit key '{unit_key}' is not one component of a key")]
    InvalidUnitKey {
        /// The unit key, for example `2024q1`.
        unit_key: String,
    },

    /// The rows do not turn into a record batch.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    FailedBatchConversion(#[from] ArrowRowError),

    /// Parquet rejects the batches or the properties of the part.
    #[error("[FailedPartEncoding] Failed to encode the part as Parquet, Reason: '{reason}'")]
    FailedPartEncoding {
        /// The text of the Parquet error.
        reason: String,
    },

    /// The store did not write the part, for example because a blob exists at its key.
    #[error("[FailedPartWrite] Failed to write the part, Caused by: {source}")]
    FailedPartWrite {
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
    const fn should_implement_auto_traits_for_part_write_error() {
        implements_auto_traits::<PartWriteError>();
    }

    #[test]
    fn should_print_the_unit_key_when_the_unit_key_is_invalid() {
        let error = PartWriteError::InvalidUnitKey {
            unit_key: "2024/q1".to_owned(),
        };

        let expected_result = "[InvalidUnitKey] Unit key '2024/q1' is not one component of a key";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_only_the_inner_error_when_the_batch_conversion_fails() {
        let error = PartWriteError::from(ArrowRowError::UnknownSchemaVersion {
            dataset: "as_is_facts",
            version: 9,
        });

        let expected_result =
            "[UnknownSchemaVersion] Dataset 'as_is_facts' has no schema version '9'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_reason_when_the_part_encoding_fails() {
        let error = PartWriteError::FailedPartEncoding {
            reason: "Invalid zstd compression level".to_owned(),
        };

        let expected_result = "[FailedPartEncoding] Failed to encode the part as Parquet, Reason: 'Invalid zstd compression level'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_the_store_error_when_the_part_write_fails() {
        let error = PartWriteError::FailedPartWrite {
            source: WriteError::conflicting_write("Blob 'a/part-0.parquet' exists"),
        };

        let expected_result = "[FailedPartWrite] Failed to write the part, Caused by: [ConflictingWrite] Write conflicts with existing data, Reason: 'Blob 'a/part-0.parquet' exists'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
