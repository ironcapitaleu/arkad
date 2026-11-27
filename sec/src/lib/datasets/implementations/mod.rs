//! # Implementations
//!
//! Provides the row types of the silver datasets, and the [`Provenance`] and [`RunColumns`] they
//! share.
//!
//! ## Modules
//!
//! - [`as_is_fact_row`]: [`AsIsFactRow`], one row of `as_is_facts`.
//! - [`candidate_row`]: [`CandidateRow`], one row of `candidates`.
//! - [`provenance`]: [`Provenance`], the raw document of a row and the run that wrote it.
//! - [`run_columns`]: [`RunColumns`], the version of the output and the run that wrote a row.

pub mod as_is_fact_row;
pub mod candidate_row;
pub mod provenance;
pub mod run_columns;

pub use as_is_fact_row::AsIsFactRow;
pub use candidate_row::CandidateRow;
pub use provenance::{Provenance, SourceKind};
pub use run_columns::RunColumns;
