//! # US GAAP Generator
//!
//! Reads the vendored FASB files of the US GAAP release for the `us_gaap` plug-in of `xbrl`.
//!
//! The build of `xbrl` parses no XML. This crate reads the vendored files with `roxmltree`, so a
//! developer can write the tables of the release as Rust source once per release.
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
