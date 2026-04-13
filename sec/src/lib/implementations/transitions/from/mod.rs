//! # Transition Source States
//!
//! This module organizes state transitions by their source states. Each submodule contains
//! transitions originating from a specific state type.

pub mod parse_company_facts;
pub mod prepare_sec_request;
pub mod validate_cik_format;
