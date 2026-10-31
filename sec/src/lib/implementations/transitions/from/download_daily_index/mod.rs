//! # Transitions from `DownloadDailyIndex`
//!
//! Holds the transitions leaving the
//! [`DownloadDailyIndex`](crate::implementations::states::extract::download_daily_index::DownloadDailyIndex)
//! state.
//!
//! ## Modules
//!
//! - [`store_daily_index`]: Transition into the state that stores the daily index.

pub mod store_daily_index;
