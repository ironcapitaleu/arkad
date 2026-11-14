//! # Disclosure Concepts
//!
//! Provides the concept enums of the disclosures. A disclosure is a set of concepts outside the
//! trees of the statements, so its concepts have no placement.
//!
//! ## Modules
//!
//! - [`per_share`]: The per-share amounts.

pub mod per_share;

pub use per_share::PerShareConcept;
