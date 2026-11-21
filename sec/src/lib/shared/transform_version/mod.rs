//! # Transform Version
//!
//! Provides [`TRANSFORM_VERSION`], the version of the silver output that this build writes.

/// The version of the silver output that this build writes.
///
/// A change of the byte form or of the rows of a fixed input needs a new version. The golden test
/// of `content_digest` holds the expected digest of its fixture unit for each version.
pub const TRANSFORM_VERSION: &str = "1";
