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
//! - [`edgar`]: The documents of EDGAR, the filing system of the SEC.

pub mod edgar;
