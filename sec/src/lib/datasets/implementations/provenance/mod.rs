//! # Provenance
//!
//! Provides [`Provenance`], the ingestion provenance and the run columns that every row of
//! `as_is_facts` and `filings` carries, and [`SourceKind`], the kind of its raw document.

use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use arrow::array::{ArrayRef, StringArray, TimestampMillisecondArray};
use arrow::datatypes::{DataType, Field};
use jiff::Timestamp;
use storage::implementations::repository::filesystem::IngestionMode;
use storage::shared::Sha256Digest;
use uuid::Uuid;

use crate::datasets::ArrowRowError;
use crate::datasets::columns::{
    BatchColumns, timestamp_array, timestamp_from_milliseconds, timestamp_type, unexpected_value,
};
use crate::shared::row_digest::ByteForm;

/// The kind of the raw document that a silver row comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceKind {
    /// A Financial Statement and Notes file.
    Notes,
    /// An XBRL instance document.
    Instance,
}

impl SourceKind {
    /// Returns the text of the `source_kind` column: `notes` or `instance`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Notes => "notes",
            Self::Instance => "instance",
        }
    }
}

impl fmt::Display for SourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The raw document of a silver row, and the run that wrote the row.
///
/// `sec/design/pipeline/silver_layer.md`, "Provenance", names the columns.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Provenance {
    /// The kind of the raw document.
    pub source_kind: SourceKind,
    /// The key of the raw document in the raw store.
    pub source_key: String,
    /// The SHA-256 of the raw document.
    pub source_sha256: Sha256Digest,
    /// The ingestion mode of the raw document, from its metadata.
    pub ingestion_mode: IngestionMode,
    /// The version of the silver output, as in
    /// [`TRANSFORM_VERSION`](crate::shared::transform_version::TRANSFORM_VERSION).
    pub transform_version: u32,
    /// The run that wrote the row.
    pub execution_id: Uuid,
    /// The time of the write. The part below one millisecond does not reach the batch.
    pub loaded_at: Timestamp,
}

impl Provenance {
    /// The names of the columns, in the order of the schema.
    const COLUMNS: [&'static str; 7] = [
        "source_kind",
        "source_key",
        "source_sha256",
        "ingestion_mode",
        "transform_version",
        "execution_id",
        "loaded_at",
    ];

    /// Returns the fields of the columns, in the order of the schema.
    pub(crate) fn fields() -> Vec<Field> {
        Self::COLUMNS
            .iter()
            .map(|name| {
                let data_type = if *name == "loaded_at" {
                    timestamp_type()
                } else {
                    DataType::Utf8
                };
                Field::new(*name, data_type, false)
            })
            .collect()
    }

    /// Returns the arrays of the columns, in the order of the schema.
    pub(crate) fn arrays(rows: &[&Self]) -> Vec<ArrayRef> {
        let text = |value: fn(&Self) -> String| -> ArrayRef {
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| value(row)),
            ))
        };
        vec![
            text(|row| row.source_kind.to_string()),
            text(|row| row.source_key.clone()),
            text(|row| row.source_sha256.to_string()),
            text(|row| row.ingestion_mode.to_string()),
            text(|row| row.transform_version.to_string()),
            text(|row| row.execution_id.to_string()),
            timestamp_array(rows.iter().map(|row| Some(row.loaded_at))),
        ]
    }

    /// Reads the columns of every row of the batch.
    ///
    /// # Errors
    ///
    /// Returns [`ArrowRowError::UnexpectedColumnValue`] if a value does not parse.
    pub(crate) fn read(columns: &BatchColumns<'_>) -> Result<Vec<Self>, ArrowRowError> {
        let [kind, key, sha256, mode, version, execution, loaded] = Self::COLUMNS;
        let kinds = columns.get::<StringArray>(kind)?;
        let keys = columns.get::<StringArray>(key)?;
        let digests = columns.get::<StringArray>(sha256)?;
        let modes = columns.get::<StringArray>(mode)?;
        let versions = columns.get::<StringArray>(version)?;
        let executions = columns.get::<StringArray>(execution)?;
        let loaded_ats = columns.get::<TimestampMillisecondArray>(loaded)?;
        (0..columns.row_count())
            .map(|row| {
                let fail = |column, reason: String| {
                    unexpected_value(columns.dataset(), column, row, reason)
                };
                Ok(Self {
                    source_kind: [SourceKind::Notes, SourceKind::Instance]
                        .into_iter()
                        .find(|candidate| candidate.as_str() == kinds.value(row))
                        .ok_or_else(|| fail(kind, "not 'notes' or 'instance'".to_owned()))?,
                    source_key: keys.value(row).to_owned(),
                    source_sha256: Sha256Digest::from_str(digests.value(row))
                        .map_err(|error| fail(sha256, error.to_string()))?,
                    ingestion_mode: [IngestionMode::Backfill, IngestionMode::Incremental]
                        .into_iter()
                        .find(|candidate| candidate.to_string() == modes.value(row))
                        .ok_or_else(|| fail(mode, "not 'backfill' or 'incremental'".to_owned()))?,
                    transform_version: versions
                        .value(row)
                        .parse()
                        .map_err(|_| fail(version, "not a base-10 number".to_owned()))?,
                    execution_id: Uuid::parse_str(executions.value(row))
                        .map_err(|error| fail(execution, error.to_string()))?,
                    loaded_at: timestamp_from_milliseconds(loaded_ats.value(row))
                        .map_err(|reason| fail(loaded, reason))?,
                })
            })
            .collect()
    }

    /// Appends the columns that enter `content_digest`, in the order of the schema.
    ///
    /// Leaves out `execution_id` and `loaded_at`, because a rebuild of the same rows changes them.
    pub(crate) fn write_digest(&self, byte_form: &mut ByteForm) {
        byte_form
            .utf8(self.source_kind.as_str())
            .utf8(&self.source_key)
            .utf8(self.source_sha256.as_str())
            .utf8(&self.ingestion_mode.to_string())
            .utf8(&self.transform_version.to_string());
    }
}

/// A provenance for the tests of the row types.
#[cfg(test)]
pub(crate) fn sample_provenance() -> Provenance {
    Provenance {
        source_kind: SourceKind::Instance,
        source_key: "sec/instances/0000320193-26-000001.xml".to_owned(),
        source_sha256: Sha256Digest::of(b"instance"),
        ingestion_mode: IngestionMode::Incremental,
        transform_version: 1,
        execution_id: Uuid::from_u128(1),
        loaded_at: Timestamp::from_millisecond(1_793_966_700_123).expect(
            "Given a hardcoded time in the range of jiff, the creation should always succeed",
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_provenance() {
        implements_auto_traits::<Provenance>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_provenance() {
        implements_value_traits::<Provenance>();
    }

    #[test]
    fn should_display_instance_for_the_instance_source_kind() {
        let source_kind = SourceKind::Instance;

        let expected_result = "instance";

        let result = source_kind.to_string();

        assert_eq!(result, expected_result);
    }
}
