//! # Validate Index Date Data
//!
//! Groups the input and output data types for the
//! [`ValidateIndexDate`](super::ValidateIndexDate) state.
//!
//! ## Modules
//!
//! - [`input`]: The [`ValidateIndexDateInput`] holding the raw date string.
//! - [`output`]: The [`ValidateIndexDateOutput`] holding the validated [`IndexDate`](crate::shared::index_date::IndexDate).

pub mod input;
pub mod output;

pub use input::ValidateIndexDateInput;
pub use output::ValidateIndexDateOutput;
