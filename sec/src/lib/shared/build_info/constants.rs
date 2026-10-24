//! # Build Info Constants
//!
//! The version and the git commit of the `sec` build.

/// The version of the `sec` crate, from `Cargo.toml`.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The git commit the `sec` crate was built from, or `unknown` for a build outside a git checkout.
pub const GIT_COMMIT: &str = env!("SEC_GIT_COMMIT");
