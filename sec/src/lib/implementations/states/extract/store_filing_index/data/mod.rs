//! # Store Filing Index Data
//!
//! Groups the output data type of the [`StoreFilingIndex`](super::StoreFilingIndex) state.
//!
//! The input data type is the generic
//! [`StoreSecResponseInput`](crate::implementations::states::extract::store_sec_response::StoreSecResponseInput).
//!
//! ## Modules
//!
//! - [`output`]: The [`StoreFilingIndexOutput`] holding the key of the stored document.

pub mod output;

pub use output::StoreFilingIndexOutput;
