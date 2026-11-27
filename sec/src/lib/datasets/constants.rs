//! # Constants
//!
//! The Arrow time zone of the timestamp columns, and the folder labels of the silver paths.

/// The Arrow time zone of every timestamp column of silver: UTC, written as an offset.
///
/// `arrow` reads an offset such as `+00:00` without its `chrono-tz` feature. It reads the name
/// `UTC` only with that feature. [ADR 0051](https://github.com/ironcapitaleu/arkad/blob/main/docs/adr/0051-utc-offset-as-arrow-time-zone.md)
/// records the choice.
pub const UTC_TIME_ZONE: &str = "+00:00";

/// The folder labels of the path of a dataset other than `silver_files`.
///
/// `DuckDB` reads a `name=value` folder as a column, so no field of the dataset has one of these
/// names.
pub const FOLDER_LABELS: [&str; 2] = ["source", "unit_key"];
