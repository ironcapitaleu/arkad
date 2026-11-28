//! # Constants
//!
//! Provides the column that [`RowFilter::EntityBuckets`](super::RowFilter::EntityBuckets) reads.

/// The column of the entity bucket of a row, a `uint8` from 0 to 63.
pub const ENTITY_BUCKET_COLUMN: &str = "entity_bucket";
