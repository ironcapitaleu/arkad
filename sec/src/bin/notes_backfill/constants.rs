//! # Constants
//!
//! The publication lag that sets the last Notes period of the backfill run, and the environment
//! variable that sets the number of Notes files in flight.

/// The number of months between today and the last Notes period that the run asks for.
///
/// The SEC posts the file of month M during month M+1. On 2026-10-29, the SEC had posted
/// `2026_09`.
pub const PUBLICATION_LAG_MONTHS: u32 = 1;

/// The environment variable that sets the number of Notes files that the run downloads at once.
pub const FILES_IN_FLIGHT_ENV_VAR: &str = "ARKAD_FILES_IN_FLIGHT";
