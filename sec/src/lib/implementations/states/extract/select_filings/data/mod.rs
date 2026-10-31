//! # Select Filings Data
//!
//! Groups the input and output data types for the
//! [`SelectFilings`](super::SelectFilings) state.
//!
//! ## Modules
//!
//! - [`input`]: The [`SelectFilingsInput`] holding the stored daily index.
//! - [`output`]: The [`SelectFilingsOutput`] holding the selected [`FilingRef`](crate::shared::filing_ref::FilingRef) list.

pub mod input;
pub mod output;

pub use input::SelectFilingsInput;
pub use output::SelectFilingsOutput;
