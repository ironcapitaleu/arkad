//! # Constants
//!
//! The size at which the unit writer flushes a part.

/// The bytes of the [`ByteForm`](crate::shared::row_digest::ByteForm) of the rows at which the
/// writer flushes a part: about 256 MB.
pub const PART_FLUSH_BYTES: usize = 256_000_000;
