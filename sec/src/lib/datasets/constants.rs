//! # Constants
//!
//! The Arrow time zone of the timestamp columns, the type of the decimal columns, and the folder
//! labels of the dataset paths.

/// The Arrow time zone of every timestamp column of the datasets: UTC, written as an offset.
///
/// `arrow` reads an offset such as `+00:00` without its `chrono-tz` feature. It reads the name
/// `UTC` only with that feature. [ADR 0051](https://github.com/ironcapitaleu/arkad/blob/main/docs/adr/0051-utc-offset-as-arrow-time-zone.md)
/// records the choice.
pub const UTC_TIME_ZONE: &str = "+00:00";

/// The folder labels of the path of a dataset other than `unit_manifests`.
///
/// `DuckDB` reads a `name=value` folder as a column, so no field of the dataset has one of these
/// names.
pub const FOLDER_LABELS: [&str; 2] = ["source", "unit_key"];

/// The folder labels of the path of `unit_manifests`.
///
/// `DuckDB` reads a `name=value` folder as a column, so no field of `unit_manifests` has one of these
/// names.
pub const UNIT_MANIFEST_FOLDER_LABELS: [&str; 2] = ["execution", "task"];

/// The digits of a decimal column, such as `value_numeric` of `as_is_facts`.
pub const DECIMAL_PRECISION: u8 = 38;

/// The digits after the point of a decimal column.
pub const DECIMAL_SCALE: i8 = 10;

/// The unscaled digits of every decimal value are below this limit: `10^38`.
pub const DECIMAL_LIMIT: u128 = 10_u128.pow(38);
