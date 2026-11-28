//! # Arrow Row
//!
//! Provides [`ArrowRow`], the trait of a row type of one silver dataset and its Arrow form.

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;

use crate::datasets::ArrowRowError;

/// A row type of one silver dataset and its Arrow form.
///
/// A trait, because each silver dataset has its own row type, and the silver writer and reader take
/// the rows of any dataset. The Rust type is the source of the schema. Each implementation builds
/// the Arrow arrays column by column, with no JSON step.
///
/// The columns of each dataset are in `sec/design/pipeline/silver_layer.md`, "Datasets".
pub trait ArrowRow: Sized {
    /// The name of the dataset, for example `as_is_facts`. The silver path and the `dataset`
    /// column of `unit_manifests` hold it.
    const DATASET: &'static str;

    /// The schema version that [`Self::to_batch`] writes.
    ///
    /// A change of the schema text that [`schema_text`](crate::datasets::schema_text()) prints adds 1.
    const SCHEMA_VERSION: u16;

    /// Returns the schema of a version, or `None` if the dataset has no such version.
    ///
    /// Each row type keeps the schema of every version that a stored unit can hold.
    fn schema(version: u16) -> Option<SchemaRef>;

    /// Builds one record batch of the rows, with the schema of [`Self::SCHEMA_VERSION`].
    ///
    /// # Errors
    ///
    /// Returns [`ArrowRowError`] in two cases:
    ///
    /// - A value has no form in its column, for example a decimal with more than 38 digits.
    /// - The values of a row contradict each other.
    fn to_batch(rows: &[Self]) -> Result<RecordBatch, ArrowRowError>;

    /// Reads the rows of a record batch that holds the schema of `version`.
    ///
    /// `version` is the `schema_version` of the unit in its `unit_manifests` row. A field that the
    /// schema of `version` lacks is an `Option` in the row type, and is `None` in each row read.
    ///
    /// # Errors
    ///
    /// Returns [`ArrowRowError`] if the dataset has no schema of `version`, if the fields of the batch
    /// differ from that schema, or if a value breaks a rule of its type.
    fn from_batch(batch: &RecordBatch, version: u16) -> Result<Vec<Self>, ArrowRowError>;
}
