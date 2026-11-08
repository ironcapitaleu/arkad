//! # Sample Taxonomy
//!
//! Provides a fake [`Taxonomy`](crate::core::taxonomy::Taxonomy) plug-in for tests that need a
//! [`Registry`](crate::core::taxonomy::registry::Registry).
//!
//! ## Modules
//!
//! - [`fake_taxonomy`]: The [`FakeTaxonomy`] that declares families and holds no tables, and the
//!   helpers [`declaration`] and [`plugin`] that build it.

pub mod fake_taxonomy;

pub use fake_taxonomy::{FakeTaxonomy, declaration, plugin};
