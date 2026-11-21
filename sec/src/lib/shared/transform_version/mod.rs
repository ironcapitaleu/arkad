//! # Transform Version
//!
//! Provides [`TRANSFORM_VERSION`], the version of the silver output that this build writes.

/// The version of the silver output that this build writes.
///
/// A counter that starts at 1. A change of the byte form or of the rows of a fixed input adds 1.
/// The silver path and the column `transform_version` hold its base-10 text. The golden test of
/// `content_digest` holds the expected digest of its fixture unit for each version.
pub const TRANSFORM_VERSION: u32 = 1;
