//! # Row Digest Traits
//!
//! The contract of a dataset row that enters the digest.
//!
//! ## Modules
//!
//! - [`digest_row`]: The [`DigestRow`] trait of a row that writes its group ID, its row key, and
//!   itself in the byte form.

pub mod digest_row;

pub use digest_row::DigestRow;
