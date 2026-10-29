//! # Extract Phase
//!
//! Provides the states of the extract phase and the super-states that drive them.
//!
//! Each super-state lives in its own module. The states stay flat in this module, because several
//! super-states share them.
//!
//! ## Modules
//!
//! - [`company_facts`]: The [`ExtractCompanyFactsSuperState`](company_facts::ExtractCompanyFactsSuperState)
//!   and its builder.
//! - [`validate_cik_format`]: Validates and normalizes the raw CIK.
//! - [`prepare_sec_request`]: Builds the request targeting the SEC API endpoint.
//! - [`execute_sec_request`]: Sends the request and captures the response.
//! - [`store_company_facts`]: Writes the raw body of the response to the store.
//! - [`validate_notes_period`]: Validates the raw period of a Notes file.
//! - [`prepare_notes_request`]: Builds the request for the Notes file of one period.
//! - [`download_notes_archive`]: Downloads the Notes file of one request.
//! - [`store_notes_archive`]: Writes the Notes file to the store.

pub mod company_facts;
pub mod download_notes_archive;
pub mod execute_sec_request;
pub mod notes_repository;
pub mod prepare_notes_request;
pub mod prepare_sec_request;
pub mod store_company_facts;
pub mod store_notes_archive;
pub mod validate_cik_format;
pub mod validate_notes_period;
