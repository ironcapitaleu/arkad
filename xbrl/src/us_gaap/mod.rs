//! # US-GAAP Taxonomy
//!
//! Concept name mappings and FASB calculation linkbase relationships
//! for the US-GAAP reporting taxonomy.
//!
//! ## Modules
//!
//! - [`mappings`]: The alias lists of the standard concepts.
//! - [`taxonomy`]: The tables of the release.
//! - [`generator`]: Reads the vendored FASB files. Only with the `generator` feature.

// rustdoc sees the generator too, so the doc check covers it. A normal build leaves it out.
#[cfg(any(doc, feature = "generator"))]
pub mod generator;
pub mod mappings;
pub mod taxonomy;
