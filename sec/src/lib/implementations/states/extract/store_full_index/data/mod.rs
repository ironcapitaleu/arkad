//! # Store Full Index Data
//!
//! Groups the output data type of the [`StoreFullIndex`](super::StoreFullIndex) state.
//!
//! The input data type is the generic
//! [`StoreSecResponseInput`](crate::implementations::states::extract::store_sec_response::StoreSecResponseInput).
//!
//! ## Modules
//!
//! - [`output`]: The [`StoreFullIndexOutput`] holding the stored full index.

pub mod output;

pub use output::StoreFullIndexOutput;
