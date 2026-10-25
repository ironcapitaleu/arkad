//! # Shared
//!
//! Provides the types that the backends and the callers of the crate both use.
//!
//! ## Modules
//!
//! - [`sha256`]: The [`Sha256Digest`] of a byte slice, and the [`InvalidSha256Digest`] for a
//!   string that is not a digest.

pub mod sha256;

pub use sha256::{InvalidSha256Digest, Sha256Digest};
