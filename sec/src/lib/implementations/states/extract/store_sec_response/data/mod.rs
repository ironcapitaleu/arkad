//! # Store SEC Response Data
//!
//! Groups the input data type of the [`StoreSecResponse`](super::StoreSecResponse) state.
//!
//! The output data type belongs to the request kind, as
//! [`StoredAs::Output`](crate::shared::request::implementations::sec_request::StoredAs::Output).
//!
//! ## Modules
//!
//! - [`input`]: The [`StoreSecResponseInput`] holding the response and the fetch details.

pub mod input;

pub use input::StoreSecResponseInput;
