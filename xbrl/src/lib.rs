//! # XBRL Domain Logic
//!
//! Parsing, concept resolution, and financial statement validation
//! for XBRL data sourced from SEC EDGAR JSON APIs.
//!
//! ## Modules
//!
//! - [`core`]: Core domain types (elements, periods, facts, confidence).
//! - [`error`]: Strongly-typed error hierarchy for parsing and validation failures.
//! - [`us_gaap`]: US-GAAP taxonomy mappings and FASB linkbase relationships.
//! - [`sec_api`]: JSON deserializers for SEC EDGAR API endpoints.

pub mod core;
pub mod error;
pub mod sec_api;
pub mod us_gaap;
