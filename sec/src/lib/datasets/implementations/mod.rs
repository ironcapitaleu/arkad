//! # Implementations
//!
//! Provides the row types of the silver datasets, and the [`Provenance`] and [`RunColumns`] they
//! share.
//!
//! ## Modules
//!
//! - [`as_is_fact_row`]: [`AsIsFactRow`], one row of `as_is_facts`.
//! - [`candidate_row`]: [`CandidateRow`], one row of `candidates`.
//! - [`finding_row`]: [`FindingRow`], one row of `findings`.
//! - [`provenance`]: [`Provenance`], the raw document of a row and the run that wrote it.
//! - [`written_unit_row`]: [`WrittenUnitRow`], one row of `silver_files`.
//! - [`run_columns`]: [`RunColumns`], the version of the output and the run that wrote a row.

pub mod as_is_fact_row;
pub mod candidate_row;
pub mod finding_row;
pub mod provenance;
pub mod run_columns;
pub mod written_unit_row;

pub use as_is_fact_row::AsIsFactRow;
pub use candidate_row::CandidateRow;
pub use finding_row::{FindingChange, FindingKind, FindingPeriod, FindingRow, FindingStep};
pub use provenance::{Provenance, SourceKind};
pub use run_columns::RunColumns;
pub use written_unit_row::{
    DroppedRows, NotesSplit, TableRows, UnitFile, UnitInput, UnitKind, WrittenUnitRow,
};
