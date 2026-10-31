//! # Transitions from `StoreDailyIndex`
//!
//! Holds the transitions leaving the
//! [`StoreDailyIndex`](crate::implementations::states::extract::store_daily_index::StoreDailyIndex)
//! state.
//!
//! ## Modules
//!
//! - [`select_filings`]: Transition into the state that selects the filings of the index.

pub mod select_filings;
