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

pub mod mappings;
pub mod table;
// `us_gaap_generator` writes this file. rustfmt must not reflow it, so a run of the generator
// gives the checked-in bytes.
#[rustfmt::skip]
pub mod taxonomy;
