//! # Transitions from `ExecuteSecRequest`
//!
//! Holds the transitions leaving the
//! [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest)
//! state.
//!
//! ## Modules
//!
//! - [`store_company_facts`]: Transition into the state that stores the Company Facts.

pub mod store_company_facts;
