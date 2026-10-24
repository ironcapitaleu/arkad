//! # Shared
//!
//! Provides the types that the backends and the callers of the crate both use.
//!
//! ## Modules
//!
//! - [`sha256`]: The [`sha256_hex`] function, the [`Sha256Digest`] that holds its result, and the
//!   [`InvalidSha256Digest`] for a string that is not a digest.

pub mod sha256;

pub use sha256::{InvalidSha256Digest, Sha256Digest, sha256_hex};
