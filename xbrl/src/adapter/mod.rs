//! # Adapters
//!
//! Provides one adapter per data source. Each adapter maps the documents of its source into
//! as-is facts and filings.
//!
//! An adapter imports [`crate::core`] and no taxonomy plug-in. The core imports no adapter, so a new
//! source needs no change to the core.
//!
//! ## Modules
//!
//! - [`edgar`]: The SEC XBRL JSON APIs of EDGAR.

pub mod edgar;
