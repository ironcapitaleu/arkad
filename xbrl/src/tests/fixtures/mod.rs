//! # Common Test Fixtures
//!
//! Reusable test doubles and data for building `xbrl` tests.
//!
//! ## Modules
//!
//! - [`sample_taxonomy`]: A fake [`Taxonomy`](crate::core::taxonomy::Taxonomy) that declares
//!   families and holds no tables.
//! - [`data`]: Real SEC files of one filing in both sources, and helpers that parse them.

pub mod data;
pub mod sample_taxonomy;
