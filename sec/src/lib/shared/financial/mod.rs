//! # Financial Domain Types
//!
//! This module contains strongly-typed domain types for financial data processing
//! within the SEC state machine library. These types provide type-safe representations
//! for SEC XBRL concepts, filing metadata, and financial observations.
//!
//! ## Modules
//! - [`mod@unit`]: Measurement units for financial data (USD, shares, etc.).
//! - [`period`]: Time period representations (instant snapshots, duration ranges).
//! - [`quarter`]: Calendar quarter representation.
//! - [`form`]: SEC filing form types (10-K, 10-Q, etc.).
//! - [`fiscal_period`]: Fiscal period identifiers (FY, Q1-Q4).
//! - [`frame`]: SEC XBRL frame identifiers for time-based data points.
//! - [`accession_number`]: SEC filing accession number newtype.
//! - [`entity_name`]: Company/entity name newtype.
//! - [`filing_source`]: Filing provenance and data lineage metadata.
//! - [`observation`]: Single measured data point with full typing and lineage.
//! - [`concept_definition`]: XBRL concept specifications for data extraction.
//! - [`company_fact`]: A company's reported data for a specific concept.
//! - [`company_data`]: Top-level container for all resolved company facts.

pub mod accession_number;
pub mod company_data;
pub mod company_fact;
pub mod concept_definition;
pub mod entity_name;
pub mod filing_source;
pub mod fiscal_period;
pub mod form;
pub mod frame;
pub mod observation;
pub mod period;
pub mod quarter;
pub mod unit;
