//! # Store Submissions Data
//!
//! Groups the output data type of the [`StoreSubmissions`](super::StoreSubmissions) state.
//!
//! The input data type is the generic
//! [`StoreSecResponseInput`](crate::implementations::states::extract::store_sec_response::StoreSecResponseInput).
//!
//! ## Modules
//!
//! - [`output`]: The [`StoreSubmissionsOutput`] holding the key of the stored document.

pub mod output;

pub use output::StoreSubmissionsOutput;
