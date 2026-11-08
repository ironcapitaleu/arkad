//! # Common Test Fixtures
//!
//! Reusable test doubles and data for building `xbrl` tests.
//!
//! ## Modules
//!
//! - [`data`]: Real SEC files of one filing in both sources, and helpers that parse them.
//! - [`sample_fact`]: Builders of as-is facts and candidates.
//! - [`sample_taxonomy`]: A fake [`Taxonomy`](crate::core::taxonomy::Taxonomy) that declares
//!   families and holds alias lists.

pub mod data;
pub mod sample_fact;
pub mod sample_taxonomy;
