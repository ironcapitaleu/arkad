//! # Unit Read Error
//!
//! Provides [`UnitReadError`], the error of the reads of the units of a dataset, such as
//! [`current_units`](crate::datasets::current_units()) and
//! [`read_rows`](crate::datasets::read_rows()).

use storage::ReadError;
use thiserror::Error;

use crate::datasets::ArrowRowError;

/// Error of a read of the `unit_manifests` list files, or of the parts of a unit.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnitReadError {
    /// The store did not list the list files.
    #[error("[FailedManifestListing] Failed to list the unit manifests, Caused by: {source}")]
    FailedManifestListing {
        /// The error of the store.
        #[source]
        source: ReadError,
    },

    /// The store did not read a list file that its listing named.
    #[error("[FailedManifestRead] Failed to read the unit manifest '{path}', Caused by: {source}")]
    FailedManifestRead {
        /// The path of the list file, for example `unit_manifests/execution=<id>/task=0-0.parquet`.
        path: String,
        /// The error of the store.
        #[source]
        source: ReadError,
    },

    /// A list file that the listing named is gone from the store.
    #[error("[MissingManifestFile] Unit manifest '{path}' was listed but not found")]
    MissingManifestFile {
        /// The path of the list file.
        path: String,
    },

    /// A list file is not Parquet, or its footer holds no valid `schema_version`.
    #[error(
        "[FailedManifestDecoding] Failed to decode the unit manifest '{path}', Reason: '{reason}'"
    )]
    FailedManifestDecoding {
        /// The path of the list file.
        path: String,
        /// The text of the Parquet error, or what the footer lacks.
        reason: String,
    },

    /// The batches of a list file do not turn into rows.
    #[error(
        "[FailedRowConversion] Failed to read the rows of the unit manifest '{path}', Caused by: {source}"
    )]
    FailedRowConversion {
        /// The path of the list file.
        path: String,
        /// The error of the row type, boxed to keep the error small.
        #[source]
        source: Box<ArrowRowError>,
    },

    /// The unit belongs to another dataset than the row type of the read.
    #[error("[UnexpectedDataset] Unit of dataset '{dataset}' read as rows of '{expected}'")]
    UnexpectedDataset {
        /// The dataset of the unit.
        dataset: String,
        /// The dataset of the row type.
        expected: &'static str,
    },

    /// The store did not read a part of a unit.
    #[error("[FailedPartRead] Failed to read the part '{path}', Caused by: {source}")]
    FailedPartRead {
        /// The path of the part, as the `files` of its unit name it.
        path: String,
        /// The error of the store.
        #[source]
        source: ReadError,
    },

    /// A part that the `files` of its unit name is not in the store.
    #[error("[MissingPartFile] Part '{path}' of a unit manifest was not found")]
    MissingPartFile {
        /// The path of the part.
        path: String,
    },

    /// A part is not Parquet, or lacks the column of the row filter.
    #[error("[FailedPartDecoding] Failed to decode the part '{path}', Reason: '{reason}'")]
    FailedPartDecoding {
        /// The path of the part.
        path: String,
        /// The text of the Parquet error, or the column that the part lacks.
        reason: String,
    },

    /// The batches of a part do not turn into rows.
    #[error(
        "[FailedPartRowConversion] Failed to read the rows of the part '{path}', Caused by: {source}"
    )]
    FailedPartRowConversion {
        /// The path of the part.
        path: String,
        /// The error of the row type, boxed to keep the error small.
        #[source]
        source: Box<ArrowRowError>,
    },
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_unit_read_error() {
        implements_auto_traits::<UnitReadError>();
    }

    #[test]
    fn should_chain_the_store_error_when_the_listing_fails() {
        let error = UnitReadError::FailedManifestListing {
            source: ReadError::missing_record("unit_manifests"),
        };

        let expected_result = "[FailedManifestListing] Failed to list the unit manifests, Caused by: [MissingRecord] Requested record 'unit_manifests' not found";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_path_when_a_listed_manifest_is_missing() {
        let error = UnitReadError::MissingManifestFile {
            path: "unit_manifests/execution=e/task=0-0.parquet".to_owned(),
        };

        let expected_result = "[MissingManifestFile] Unit manifest 'unit_manifests/execution=e/task=0-0.parquet' was listed but not found";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_reason_when_a_manifest_does_not_decode() {
        let error = UnitReadError::FailedManifestDecoding {
            path: "unit_manifests/execution=e/task=0-0.parquet".to_owned(),
            reason: "the footer holds no valid schema_version".to_owned(),
        };

        let expected_result = "[FailedManifestDecoding] Failed to decode the unit manifest 'unit_manifests/execution=e/task=0-0.parquet', Reason: 'the footer holds no valid schema_version'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_the_row_error_when_the_rows_do_not_convert() {
        let error = UnitReadError::FailedRowConversion {
            path: "unit_manifests/execution=e/task=0-0.parquet".to_owned(),
            source: Box::new(ArrowRowError::UnknownSchemaVersion {
                dataset: "unit_manifests",
                version: 9,
            }),
        };

        let expected_result = "[FailedRowConversion] Failed to read the rows of the unit manifest 'unit_manifests/execution=e/task=0-0.parquet', Caused by: [UnknownSchemaVersion] Dataset 'unit_manifests' has no schema version '9'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_both_datasets_when_a_unit_is_read_as_another_dataset() {
        let error = UnitReadError::UnexpectedDataset {
            dataset: "filings".to_owned(),
            expected: "candidates",
        };

        let expected_result =
            "[UnexpectedDataset] Unit of dataset 'filings' read as rows of 'candidates'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_reason_when_a_part_does_not_decode() {
        let error = UnitReadError::FailedPartDecoding {
            path: "candidates/unit_key=2024q1/part-0.parquet".to_owned(),
            reason: "the part has no uint8 column entity_bucket".to_owned(),
        };

        let expected_result = "[FailedPartDecoding] Failed to decode the part 'candidates/unit_key=2024q1/part-0.parquet', Reason: 'the part has no uint8 column entity_bucket'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
