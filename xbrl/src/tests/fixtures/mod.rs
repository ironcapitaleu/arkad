//! # Common Test Fixtures
//!
//! Reusable test doubles for building `xbrl` tests.
//!
//! ## Modules
//!
//! - [`sample_fact`]: Builders of as-is facts and candidates.
//! - [`sample_taxonomy`]: A fake [`Taxonomy`](crate::core::taxonomy::Taxonomy) that declares
//!   families and holds alias lists.

pub mod sample_fact;
pub mod sample_taxonomy;
