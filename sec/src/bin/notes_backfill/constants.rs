//! # Constants
//!
//! The publication lag that sets the last Notes period of the backfill run.

/// The number of months between today and the last Notes period that the run downloads.
///
/// The SEC posts the file of month M during month M+1. On 2026-10-29, `2026_09` was posted.
pub const PUBLICATION_LAG_MONTHS: u32 = 2;
