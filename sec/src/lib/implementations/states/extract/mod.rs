//! # Extract Phase
//!
//! Provides the states of the extract phase and the super-states that drive them.
//!
//! Each super-state lives in its own module. The states stay flat in this module, because later
//! super-states reuse them.
//!
//! ## Modules
//!
//! - [`company_facts`]: The [`ExtractCompanyFactsSuperState`](company_facts::ExtractCompanyFactsSuperState)
//!   and its builder.
//! - [`notes`]: The [`ExtractNotesSuperState`](notes::ExtractNotesSuperState) and its builder.
//! - [`daily_index`]: The [`DetectFilingsSuperState`](daily_index::DetectFilingsSuperState) and its
//!   builder.
//! - [`submissions`]: The [`ExtractSubmissionsSuperState`](submissions::ExtractSubmissionsSuperState)
//!   and its builder.
//! - [`validate_cik_format`]: Validates and normalizes the raw CIK.
//! - [`prepare_sec_request`]: Builds the request targeting the SEC API endpoint.
//! - [`execute_sec_request`]: Sends the request and captures the response.
//! - [`store_company_facts`]: Writes the raw body of the response to the store.
//! - [`validate_notes_period`]: Validates the raw period of a Notes file.
//! - [`store_notes_archive`]: Writes the Notes file to the store.
//! - [`validate_index_date`]: Validates the raw date of an EDGAR daily index.
//! - [`download_daily_index`]: Downloads the daily index of one date, with the rule for a missing index.
//! - [`store_daily_index`]: Writes the daily index to the store.
//! - [`select_filing_documents`]: Selects the XBRL instance and the label file of one filing from its folder index.
//! - [`select_filings`]: Selects the filings to fetch from the daily index.
//! - [`store_submissions`]: Writes the `submissions` JSON of one company to the store.
//! - [`store_filing_documents`]: Writes the XBRL instance and the label file of one filing to the store.

pub mod company_facts;
pub mod daily_index;
pub mod download_daily_index;
pub mod execute_sec_request;
pub mod notes;
pub mod prepare_sec_request;
pub mod select_filing_documents;
pub mod select_filings;
pub mod store_company_facts;
pub mod store_daily_index;
pub mod store_filing_documents;
pub mod store_notes_archive;
pub mod store_submissions;
pub mod submissions;
pub mod validate_cik_format;
pub mod validate_index_date;
pub mod validate_notes_period;
