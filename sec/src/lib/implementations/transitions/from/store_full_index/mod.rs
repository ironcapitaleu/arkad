//! # Transitions from `StoreFullIndex`
//!
//! Holds the transitions leaving the
//! [`StoreFullIndex`](crate::implementations::states::extract::store_full_index::StoreFullIndex)
//! state.
//!
//! ## Modules
//!
//! - [`select_full_index_filings`]: Transition into the state that selects the filings of a date
//!   range from the index.

pub mod select_full_index_filings;
