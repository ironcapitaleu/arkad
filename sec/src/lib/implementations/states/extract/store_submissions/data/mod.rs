//! # Store Submissions Data
//!
//! Groups the input and output data types for the
//! [`StoreSubmissions`](super::StoreSubmissions) state.
//!
//! The state consumes an SEC response and the fetch that produced it, and produces the key of the
//! stored document.
//!
//! ## Modules
//!
//! - [`input`]: The [`StoreSubmissionsInput`] holding the response and the fetch details.
//! - [`output`]: The [`StoreSubmissionsOutput`] holding the key of the stored document.

pub mod input;
pub mod output;

pub use input::StoreSubmissionsInput;
pub use output::StoreSubmissionsOutput;
