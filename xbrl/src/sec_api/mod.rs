//! # SEC EDGAR API Parsers
//!
//! Maps the JSON bodies of the SEC XBRL APIs into as-is facts and filings.
//!
//! `sec_api` is a source adapter. It imports [`crate::core`] and no plug-in. The `sec` crate fetches
//! the bodies, so this module has no network dependency.
//!
//! ## Modules
//!
//! - [`company_facts`]: The Company Facts API. Implemented.
//! - [`company_concept`]: The Company Concept API. Placeholder.
//! - [`frames`]: The Frames API. Placeholder.
//! - [`frame`]: The `frame` field of a data point.
//! - [`quarter`]: The calendar quarter of a frame.

pub mod company_concept;
pub mod company_facts;
pub mod frame;
pub mod frames;
pub mod quarter;
