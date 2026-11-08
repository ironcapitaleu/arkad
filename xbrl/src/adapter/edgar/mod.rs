//! # EDGAR
//!
//! Maps the JSON bodies of the SEC XBRL APIs into as-is facts and filings.
//!
//! EDGAR is the filing system of the SEC. The `sec` crate fetches the bodies, so this module has no
//! network dependency.
//!
//! ## Modules
//!
//! - [`company_facts`]: The Company Facts API.
//! - [`company_concept`]: The Company Concept API.
//! - [`frames`]: The Frames API.
//! - [`frame`]: The `frame` field of a data point.
//! - [`quarter`]: The calendar quarter of a frame.

pub mod company_concept;
pub mod company_facts;
pub mod frame;
pub mod frames;
pub mod quarter;
