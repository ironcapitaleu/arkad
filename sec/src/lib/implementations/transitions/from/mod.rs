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

pub mod execute_sec_request;
pub mod parse_company_facts;
pub mod prepare_sec_request;
pub mod validate_cik_format;
pub mod validate_notes_period;
