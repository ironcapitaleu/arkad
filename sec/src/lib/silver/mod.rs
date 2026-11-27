//! # Silver
//!
//! Provides [`ArrowRow`], the trait of a row type of one silver dataset and its Arrow form,
//! and the row types of the datasets.
//!
//! The Rust row types are the source of each schema. Each row type builds its Arrow arrays column
//! by column, and reads them back. [`schema_text()`] prints a schema for the snapshot tests.
//!
//! ## Modules
//!
//! - [`traits`]: The [`ArrowRow`] contract of a row type.
//! - [`implementations`]: The row types, such as [`AsIsFactRow`], and their [`Provenance`].
//! - [`silver_error`]: The [`SilverError`] of a row type.
//! - [`schema_text`](mod@schema_text): The text form of a schema.
//! - [`constants`]: The Arrow time zone and the folder labels of the silver paths.
//!
//! ## See Also
//!
//! - `sec/design/pipeline/silver_layer.md`, "Datasets": the columns of each dataset.
//! - `sec/design/pipeline/silver_writer_and_jobs.md`, "`ArrowRow` and `DigestRow`" and
//!   "Schema Tests".

mod columns;
pub mod constants;
pub mod implementations;
pub mod schema_text;
pub mod silver_error;
pub mod traits;

pub use constants::{FOLDER_LABELS, UTC_TIME_ZONE};
pub use implementations::{AsIsFactRow, Provenance, SourceKind};
pub use schema_text::schema_text;
pub use silver_error::SilverError;
pub use traits::ArrowRow;
