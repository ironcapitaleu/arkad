//! # Select Full Index Filings Data
//!
//! Groups the input data type of the [`SelectFullIndexFilings`](super::SelectFullIndexFilings)
//! state.
//!
//! The output data type is the
//! [`SelectFilingsOutput`](crate::implementations::states::extract::select_filings::SelectFilingsOutput)
//! of the daily index path, because both states output the same list of filings.
//!
//! ## Modules
//!
//! - [`input`]: The [`SelectFullIndexFilingsInput`] holding the stored full index.

pub mod input;

pub use input::SelectFullIndexFilingsInput;
