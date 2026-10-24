//! # Shared
//!
//! Provides the types that more than one backend, or a caller of the crate, uses.
//!
//! ## Modules
//!
//! - [`sha256`]: The [`sha256_hex`] function and the [`Sha256Digest`] that holds its result.

pub mod sha256;

pub use sha256::{Sha256Digest, sha256_hex};
