//! # Datasets
//!
//! Provides [`ArrowRow`], the trait of a row type of one dataset and its Arrow form, the row types
//! of the datasets, the [`UnitWriter`] of a unit and its parts, [`write_task_manifest()`], which
//! makes the units of a task visible, and [`current_units()`], which reads them.
//!
//! The Rust row types are the source of each schema. Each row type builds its Arrow arrays column
//! by column, and reads them back. [`schema_text()`] prints a schema for the snapshot tests.
//!
//! ## Modules
//!
//! - [`traits`]: The [`ArrowRow`] contract of a row type, and the [`PartOrder`] of its rows in a
//!   part.
//! - [`implementations`]: The row types, such as [`AsIsFactRow`], [`CandidateRow`],
//!   [`FilingRow`], [`FindingRow`], and [`UnitManifestRow`], and their [`Provenance`] and [`RunColumns`].
//! - [`arrow_row_error`]: The [`ArrowRowError`] of a row type.
//! - [`part_writer`]: [`write_part()`] and the [`PartLocation`] of a part.
//! - [`part_write_error`]: The [`PartWriteError`] of [`write_part()`].
//! - [`unit_writer`]: The [`UnitWriter`], its [`UnitSpec`], and [`input_digest()`].
//! - [`task_manifest_writer`]: [`write_task_manifest()`] and the [`TaskLocation`] of a list file.
//! - [`unit_write_error`]: The [`UnitWriteError`] of the writers of a unit and of a list file.
//! - [`unit_reader`]: [`current_units()`], the current unit of each unit key.
//! - [`unit_read_error`]: The [`UnitReadError`] of [`current_units()`].
//! - [`schema_text`](mod@schema_text): The text form of a schema.
//! - [`constants`]: The Arrow time zone, the type of the decimal columns, and the folder labels of
//!   the dataset paths.

pub mod arrow_row_error;
mod columns;
pub mod constants;
pub mod implementations;
pub mod part_write_error;
pub mod part_writer;
pub mod schema_text;
pub mod task_manifest_writer;
pub mod traits;
pub mod unit_read_error;
pub mod unit_reader;
pub mod unit_write_error;
pub mod unit_writer;

pub use arrow_row_error::ArrowRowError;
pub use constants::{FOLDER_LABELS, UNIT_MANIFEST_FOLDER_LABELS, UTC_TIME_ZONE};
pub use implementations::{
    AsIsFactRow, CandidateRow, FilingRow, FindingChange, FindingKind, FindingPeriod, FindingRow,
    FindingStep, Provenance, RunColumns, SicSource, SourceKind, UnitManifestRow,
};
pub use part_write_error::PartWriteError;
pub use part_writer::{PartLocation, TaskAttempt, write_part};
pub use schema_text::schema_text;
pub use task_manifest_writer::{TaskLocation, write_task_manifest};
pub use traits::{ArrowRow, PartOrder};
pub use unit_read_error::UnitReadError;
pub use unit_reader::{CurrentUnits, current_units};
pub use unit_write_error::UnitWriteError;
pub use unit_writer::{UnitSpec, UnitStart, UnitWriter, WrittenUnit, input_digest};
