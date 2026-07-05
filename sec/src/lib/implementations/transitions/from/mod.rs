//! # Transitions by Source State
//!
//! Groups the pipeline's transitions by the state each one leaves from; every child module holds
//! the conversions originating from one state.
//!
//! ## Modules
//!
//! - [`validate_cik_format`]: Transitions out of the CIK-validation state.
//! - [`prepare_sec_request`]: Transitions out of the request-preparation state.
//! - [`parse_company_facts`]: Transitions out of the Company Facts parsing state.

pub mod parse_company_facts;
pub mod prepare_sec_request;
pub mod validate_cik_format;
