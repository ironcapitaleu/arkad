//! # US GAAP Generator
//!
//! Reads the vendored FASB files of the US GAAP release, so a developer can write the tables of the
//! release as Rust source once per release.
//!
//! The build of `xbrl` parses no XML for the tables. This module is compiled only with the
//! `generator` feature, so a normal build of `xbrl` or of a crate that depends on it leaves it out.
//!
//! ## Modules
//!
//! - [`error`]: The error of a read.
//! - [`linkbase`]: Reads labels and calculation arcs.
//! - [`schema`]: Reads the concepts of the element schema.
//!
//! ## See Also
//!
//! - `xbrl/DESIGN.md`, "Taxonomy Data in Code".

pub mod error;
pub mod linkbase;
pub mod schema;
