//! # Transitions from `ValidateCikFormat`
//!
//! Holds the transitions leaving the
//! [`ValidateCikFormat`](crate::implementations::states::extract::validate_cik_format::ValidateCikFormat)
//! state.
//!
//! ## Modules
//!
//! - [`prepare_sec_request`]: Transition into the request-preparation state.

pub mod prepare_sec_request;
