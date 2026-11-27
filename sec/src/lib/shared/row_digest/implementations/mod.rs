//! # Row Digest Implementations
//!
//! The concrete types that write the byte form of a row.
//!
//! ## Modules
//!
//! - [`byte_form`]: The [`ByteForm`] buffer, with one method per Arrow type
//!   of the contract.

pub mod byte_form;

pub use byte_form::ByteForm;
