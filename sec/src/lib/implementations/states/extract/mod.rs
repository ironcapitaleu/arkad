//! # Extract State Module
//!
//! This module contains state implementations responsible for the extraction and initial validation of raw SEC filings data.
//! It provides the entry point for the Extract phase in the SEC state machine ETL workflow.
//!
//! ## Submodules
//! - [`validate_cik_format`]: Implements states for validating and extracting CIK (Central Index Key) information from SEC filings, including format checks and normalization routines.
//!
//! The extract states are designed to be composed within state machines, enabling robust, testable, and extensible data ingestion pipelines for SEC filings processing.
//!
//! See the documentation for each submodule for details on their specific responsibilities and usage.

pub mod validate_cik_format;
#[allow(dead_code)]
struct ExtractState {}
