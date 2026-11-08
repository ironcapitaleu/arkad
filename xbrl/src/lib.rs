//! # XBRL Domain Logic
//!
//! Provides the fact model of reported financial data, the resolution of facts to standard
//! concepts, and financial statement validation.
//!
//! The core knows no taxonomy and no source. A taxonomy plug-in, such as [`us_gaap`], brings the
//! knowledge of one taxonomy. An adapter, such as [`adapter::edgar`], maps one data source into as-is
//! facts.
//!
//! ## Modules
//!
//! - [`adapter`]: One adapter per data source. Each maps the documents of its source into as-is
//!   facts and filings.
//! - [`core`]: The fact model and the types every source and plug-in shares.
//! - [`error`]: Strongly-typed error hierarchy for parsing and validation failures.
//! - [`us_gaap`]: US-GAAP taxonomy mappings and FASB linkbase relationships.

pub mod adapter;
pub mod core;
pub mod error;
pub mod us_gaap;

#[cfg(test)]
pub mod tests;
