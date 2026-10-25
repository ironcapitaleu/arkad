//! # Store Company Facts Data
//!
//! Groups the input and output data types for the
//! [`StoreCompanyFacts`](super::StoreCompanyFacts) state.
//!
//! The state consumes an SEC response and the fetch that produced it, and produces the key of the
//! stored document.
//!
//! ## Modules
//!
//! - [`input`]: The [`StoreCompanyFactsInput`] holding the response and the fetch details.
//! - [`output`]: The [`StoreCompanyFactsOutput`] holding the key of the stored document.

pub mod input;
pub mod output;

pub use input::StoreCompanyFactsInput;
pub use output::StoreCompanyFactsOutput;
