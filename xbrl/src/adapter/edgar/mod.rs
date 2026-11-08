//! # EDGAR
//!
//! Maps the rows of the SEC Notes data set and the JSON bodies of the SEC XBRL APIs into as-is facts
//! and filings.
//!
//! EDGAR is the filing system of the SEC. The `sec` crate fetches the documents and the transform
//! splits them, so this module reads no file and has no network dependency.
//!
//! ## Modules
//!
//! - [`company_facts`]: The Company Facts API.
//! - [`company_concept`]: The Company Concept API.
//! - [`frames`]: The Frames API.
//! - [`notes`]: The rows of one filing of the Notes data set.
//! - [`frame`]: The `frame` field of a data point.
//! - [`quarter`]: The calendar quarter of a frame.

pub mod company_concept;
pub mod company_facts;
pub mod frame;
pub mod frames;
pub mod notes;
pub mod quarter;
