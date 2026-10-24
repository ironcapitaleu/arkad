//! # Shared
//!
//! Provides the types that the backends and the callers of the crate both use.
//!
//! ## Modules
//!
//! - [`sha256`]: The [`sha256_hex`] function and the [`Sha256Digest`] that holds its result.

pub mod sha256;

pub use sha256::{Sha256Digest, sha256_hex};
