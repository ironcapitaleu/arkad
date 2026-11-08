//! # XBRL Domain Logic
//!
//! Provides the fact model of reported financial data, the resolution of facts to standard
//! concepts, and financial statement validation.
//!
//! The core knows no taxonomy and no source. A taxonomy plug-in, such as [`us_gaap`], brings the
//! knowledge of one taxonomy. A source adapter, such as [`sec_api`], maps one source into as-is
//! facts.
//!
//! ## Modules
//!
//! - [`core`]: The fact model and the types every source and plug-in shares.
//! - [`error`]: Strongly-typed error hierarchy for parsing and validation failures.
//! - [`us_gaap`]: US-GAAP taxonomy mappings and FASB linkbase relationships.
//! - [`sec_api`]: Maps the SEC XBRL JSON APIs into as-is facts and filings.

pub mod core;
pub mod error;
pub mod sec_api;
pub mod us_gaap;
