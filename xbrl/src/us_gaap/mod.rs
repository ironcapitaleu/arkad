//! # US-GAAP Taxonomy
//!
//! Concept name mappings and FASB calculation linkbase relationships
//! for the US-GAAP reporting taxonomy.
//!
//! ## Modules
//!
//! - [`mappings`]: The alias lists of the standard concepts.
//! - [`taxonomy`]: The tables of the release.
//! - `generator`: Reads the vendored FASB files. Only with the `generator` feature.

#[cfg(feature = "generator")]
pub mod generator;
pub mod mappings;
pub mod taxonomy;
