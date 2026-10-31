//! # Raw Source Implementations
//!
//! One marker type per SEC file, each implementing the [`RawSource`](super::traits::RawSource)
//! contract.
//!
//! ## Modules
//!
//! - [`raw_company_facts`]: The [`RawCompanyFacts`] marker for the Company Facts JSON.
//! - [`raw_daily_index`]: The [`RawDailyIndex`] marker for the EDGAR daily index of one day.
//! - [`raw_notes_archive`]: The [`RawNotesArchive`] marker for a Financial Statement and Notes ZIP.
//! - [`raw_submissions`]: The [`RawSubmissions`] marker for the `submissions` JSON.

pub mod raw_company_facts;
pub mod raw_daily_index;
pub mod raw_notes_archive;
pub mod raw_submissions;

pub use raw_company_facts::RawCompanyFacts;
pub use raw_daily_index::RawDailyIndex;
pub use raw_notes_archive::RawNotesArchive;
pub use raw_submissions::RawSubmissions;
