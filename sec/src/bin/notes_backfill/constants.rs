//! # Constants
//!
//! The publication lag that sets the last Notes period of the backfill run.

/// The number of months between today and the last Notes period that the run downloads.
///
/// The SEC publishes the file of month M some days into month M+1, and the day varies. On
/// 2026-10-29, `2026_09` returned HTTP 200 and `2026_10` returned HTTP 404. A lag of two months
/// never asks for a file that the SEC has not published yet.
pub const PUBLICATION_LAG_MONTHS: u32 = 2;
