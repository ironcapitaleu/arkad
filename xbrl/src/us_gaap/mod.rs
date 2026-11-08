//! # US-GAAP Taxonomy
//!
//! Concept name mappings and the tables of the FASB US GAAP release for the US-GAAP reporting
//! taxonomy.
//!
//! ## Modules
//!
//! - [`mappings`]: The alias lists of the standard concepts.
//! - [`table`]: The row types of the generated tables.
//! - [`taxonomy`]: The generated tables of the release: calculation arcs and element data.
//! - [`generator`]: Writes [`taxonomy`] from the vendored FASB files. Only with the `generator`
//!   feature.

// rustdoc sees the generator too, so the doc check covers it. A normal build leaves it out.
#[cfg(any(doc, feature = "generator"))]
pub mod generator;
pub mod mappings;
pub mod table;
// The generator writes this file. rustfmt must not reflow it, so a run of the generator gives the
// checked-in bytes.
#[rustfmt::skip]
pub mod taxonomy;
