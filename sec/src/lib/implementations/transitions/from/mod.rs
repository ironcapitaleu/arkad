//! # Transitions by Source State
//!
//! Groups the pipeline's transitions by the state each one leaves from; every child module holds
//! the conversions originating from one state.
//!
//! ## Modules
//!
//! - [`validate_cik_format`]: Transitions out of the CIK-validation state.
//! - [`prepare_sec_request`]: Transitions out of the request-preparation state.
//! - [`execute_sec_request`]: Transitions out of the request-execution state.
//! - [`parse_company_facts`]: Transitions out of the Company Facts parsing state.
//! - [`validate_notes_period`]: Transitions out of the Notes period validation state.
//! - [`validate_index_date`]: Transitions out of the index date validation state.
//! - [`download_daily_index`]: Transitions out of the daily index download state.
//! - [`store_daily_index`]: Transitions out of the daily index store state.
//! - [`store_filing_index`]: Transitions out of the folder index store state.
//! - [`select_filing_documents`]: Transitions out of the filing documents selection state.

pub mod download_daily_index;
pub mod execute_sec_request;
pub mod parse_company_facts;
pub mod prepare_sec_request;
pub mod select_filing_documents;
pub mod store_daily_index;
pub mod store_filing_index;
pub mod validate_cik_format;
pub mod validate_index_date;
pub mod validate_notes_period;
