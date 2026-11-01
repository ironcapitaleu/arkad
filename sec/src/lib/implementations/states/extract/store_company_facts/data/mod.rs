//! # Store Company Facts Data
//!
//! Holds the output data type of the [`StoreCompanyFacts`](super::StoreCompanyFacts) state. The
//! input data type is the generic
//! [`StoreSecResponseInput`](crate::implementations::states::extract::store_sec_response::StoreSecResponseInput).
//!
//! ## Modules
//!
//! - [`output`]: The [`StoreCompanyFactsOutput`] holding the key of the stored document.

pub mod output;

pub use output::StoreCompanyFactsOutput;
