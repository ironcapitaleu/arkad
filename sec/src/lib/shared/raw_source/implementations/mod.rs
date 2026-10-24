//! # Raw Source Implementations
//!
//! One marker type per SEC file, each implementing the [`RawSource`](super::traits::RawSource)
//! contract.
//!
//! ## Modules
//!
//! - [`raw_company_facts`]: The [`RawCompanyFacts`] marker for the Company Facts JSON.

pub mod raw_company_facts;

pub use raw_company_facts::RawCompanyFacts;
