//! # Constants
//!
//! The row limit of a row group, the zstd level, and the footer keys of a part.

/// The most rows of one row group.
pub const ROW_GROUP_ROWS: usize = 100_000;

/// The zstd level of each column chunk.
pub const ZSTD_LEVEL: i32 = 3;

/// The footer key of the dataset, for example `as_is_facts`.
pub const FOOTER_DATASET: &str = "dataset";

/// The footer key of the schema version of the rows.
pub const FOOTER_SCHEMA_VERSION: &str = "schema_version";

/// The footer key of the unit key, for example `2024q1`.
pub const FOOTER_UNIT_KEY: &str = "unit_key";

/// The footer key of the version of the transform output.
pub const FOOTER_TRANSFORM_VERSION: &str = "transform_version";
