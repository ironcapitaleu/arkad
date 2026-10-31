//! # Constants
//!
//! The publication lag that sets the last Notes period of the backfill run.

/// The number of months between today and the last Notes period that the run asks for.
///
/// The SEC posts the file of month M during month M+1. On 2026-10-29, the SEC had posted
/// `2026_09`. Early in a month, the file of the last month can be absent, and the run reports it as
/// unpublished.
pub const PUBLICATION_LAG_MONTHS: u32 = 1;
