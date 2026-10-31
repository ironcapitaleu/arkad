//! # Transitions from `ValidateIndexDate`
//!
//! Holds the transitions leaving the
//! [`ValidateIndexDate`](crate::implementations::states::extract::validate_index_date::ValidateIndexDate)
//! state.
//!
//! ## Modules
//!
//! - [`prepare_daily_index_request`]: Transition into the state that builds the daily index
//!   request.

pub mod prepare_daily_index_request;
