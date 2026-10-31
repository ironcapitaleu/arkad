//! # Transitions from `PrepareSecRequest`
//!
//! Holds the transitions leaving the
//! [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest)
//! state.
//!
//! ## Modules
//!
//! - [`execute_sec_request`]: Transition into the request-execution state.
//! - [`download_daily_index`]: Transition of the daily index kind into the state that downloads
//!   the daily index.

pub mod download_daily_index;
pub mod execute_sec_request;
