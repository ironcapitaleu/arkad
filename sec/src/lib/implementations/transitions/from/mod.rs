//! # Transition Source States
//!
//! This module organizes state transitions by their source states. Each submodule contains
//! transitions originating from a specific state type.

pub mod validate_cik_format;
pub mod prepare_sec_request;
pub mod execute_sec_request;
