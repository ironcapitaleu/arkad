//! # Implementations
//!
//! Provides the row types of the silver datasets, and the [`Provenance`] columns they share.
//!
//! ## Modules
//!
//! - [`as_is_fact_row`]: [`AsIsFactRow`], one row of `as_is_facts`.
//! - [`provenance`]: [`Provenance`], the raw document of a row and the run that wrote it.

pub mod as_is_fact_row;
pub mod provenance;

pub use as_is_fact_row::AsIsFactRow;
pub use provenance::{Provenance, SourceKind};
