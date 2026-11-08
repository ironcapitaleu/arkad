//! # US GAAP Generator
//!
//! Reads the vendored FASB files of a US GAAP release, so a developer can write its tables as Rust
//! source.
//!
//! The build of `xbrl` parses no XML for the tables. This module is compiled only with the
//! `generator` feature, so a normal build of `xbrl` or of a crate that depends on it leaves it out.
//!
//! ## Modules
//!
//! - [`error`]: The error of a read.
//! - [`schema`]: Reads the concepts of the element schema.
//!
//! ## See Also
//!
//! - `xbrl/DESIGN.md`, "Taxonomy Data in Code".

pub mod error;
pub mod schema;
