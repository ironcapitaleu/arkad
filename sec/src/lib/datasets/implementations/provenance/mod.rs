//! # Provenance
//!
//! Provides [`Provenance`], the ingestion provenance and the [`RunColumns`] that every row of
//! `as_is_facts` and `filings` carries, and [`SourceKind`], the kind of its raw document.

use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use arrow::array::{ArrayRef, StringArray};
use arrow::datatypes::{DataType, Field};
use storage::implementations::repository::filesystem::IngestionMode;
use storage::shared::Sha256Digest;

use crate::datasets::columns::{BatchColumns, ingestion_mode_from_name, unexpected_value};
use crate::datasets::{ArrowRowError, RunColumns};
use crate::shared::row_digest::ByteForm;

/// The kind of the raw document that a dataset row comes from.
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

    /// Returns the source kind whose text is `text`, or `None` if no kind has it.
    #[must_use]
    pub fn from_name(text: &str) -> Option<Self> {
        [Self::Notes, Self::Instance]
            .into_iter()
            .find(|kind| kind.as_str() == text)
    }
}

impl fmt::Display for SourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The raw document of a dataset row, and the run that wrote the row.
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
    /// The version of the transform output and the run that wrote the row.
    pub run: RunColumns,
}

impl Provenance {
    /// The names of the source columns, in the order of the schema. The run columns follow them.
    const COLUMNS: [&'static str; 4] = [
        "source_kind",
        "source_key",
        "source_sha256",
        "ingestion_mode",
    ];

    /// Returns the fields of the columns, in the order of the schema.
    pub(crate) fn fields() -> Vec<Field> {
        let mut fields: Vec<Field> = Self::COLUMNS
            .iter()
            .map(|name| Field::new(*name, DataType::Utf8, false))
            .collect();
        fields.extend(RunColumns::fields());
        fields
    }

    /// Returns the arrays of the columns, in the order of the schema.
    pub(crate) fn arrays(rows: &[&Self]) -> Vec<ArrayRef> {
        let text = |value: fn(&Self) -> String| -> ArrayRef {
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| value(row)),
            ))
        };
        let runs: Vec<RunColumns> = rows.iter().map(|row| row.run).collect();
        let mut arrays = vec![
            text(|row| row.source_kind.to_string()),
            text(|row| row.source_key.clone()),
            text(|row| row.source_sha256.to_string()),
            text(|row| row.ingestion_mode.to_string()),
        ];
        arrays.extend(RunColumns::arrays(&runs));
        arrays
    }

    /// Reads the columns of every row of the batch.
    ///
    /// # Errors
    ///
    /// Returns [`ArrowRowError::UnexpectedColumnValue`] if a value does not parse.
    pub(crate) fn read(columns: &BatchColumns<'_>) -> Result<Vec<Self>, ArrowRowError> {
        let [kind, key, sha256, mode] = Self::COLUMNS;
        let kinds = columns.get::<StringArray>(kind)?;
        let keys = columns.get::<StringArray>(key)?;
        let digests = columns.get::<StringArray>(sha256)?;
        let modes = columns.get::<StringArray>(mode)?;
        let runs = RunColumns::read(columns)?;
        runs.into_iter()
            .enumerate()
            .map(|(row, run)| {
                let fail = |column, reason: String| {
                    unexpected_value(columns.dataset(), column, row, reason)
                };
                Ok(Self {
                    source_kind: SourceKind::from_name(kinds.value(row))
                        .ok_or_else(|| fail(kind, "not 'notes' or 'instance'".to_owned()))?,
                    source_key: keys.value(row).to_owned(),
                    source_sha256: Sha256Digest::from_str(digests.value(row))
                        .map_err(|error| fail(sha256, error.to_string()))?,
                    ingestion_mode: ingestion_mode_from_name(modes.value(row))
                        .ok_or_else(|| fail(mode, "not 'backfill' or 'incremental'".to_owned()))?,
                    run,
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
            .utf8(&self.ingestion_mode.to_string());
        self.run.write_digest(byte_form);
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
        run: crate::datasets::implementations::run_columns::sample_run_columns(),
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

    #[test]
    fn should_read_every_source_kind_back_from_its_text() {
        let texts = ["notes", "instance"];

        let expected_result = [Some(SourceKind::Notes), Some(SourceKind::Instance)];

        let result = texts.map(SourceKind::from_name);

        assert_eq!(result, expected_result);
    }
}
