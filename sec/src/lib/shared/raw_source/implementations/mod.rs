//! # Raw Source Implementations
//!
//! One marker type per SEC file, each implementing the [`RawSource`](super::traits::RawSource)
//! contract.
//!
//! ## Modules
//!
//! - [`raw_company_facts`]: The [`RawCompanyFacts`] marker for the Company Facts JSON.
//! - [`raw_notes_archive`]: The [`RawNotesArchive`] marker for a Financial Statement and Notes ZIP.

pub mod raw_company_facts;
pub mod raw_notes_archive;

pub use raw_company_facts::RawCompanyFacts;
pub use raw_notes_archive::RawNotesArchive;
