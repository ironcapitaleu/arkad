//! # SEC EDGAR API Parsers
//!
//! JSON deserializers for SEC XBRL JSON API endpoints.
//! Currently only [`company_facts`] is implemented. The other two
//! modules are placeholders for future implementation.

pub mod company_concept;
pub mod company_facts;
pub mod frames;
