//! # Constants
//!
//! The publication lag that sets the last Notes period of the backfill run, the cap on unpublished
//! periods, and the default number of Notes files in flight.

/// The number of months between today and the last Notes period that the run asks for.
///
/// The SEC posts the file of month M during month M+1. On 2026-10-29, the SEC had posted
/// `2026_09`.
pub const PUBLICATION_LAG_MONTHS: u32 = 1;

/// The most periods at the end of a run that the run summary counts as unpublished.
///
/// The run asks for periods up to last month, so only the last period can lack a file. The cap
/// keeps a run in which the SEC refuses every request with 403 from counting as a success.
pub const MAX_UNPUBLISHED: usize = 1;

/// The environment variable that sets the number of Notes files that the run downloads at once.
pub const FILES_IN_FLIGHT_ENV_VAR: &str = "ARKAD_FILES_IN_FLIGHT";

/// The number of Notes files that the run downloads at once if `ARKAD_FILES_IN_FLIGHT` is unset or
/// empty.
///
/// The rate limiter limits requests per second, not files in memory, so this cap bounds memory.
/// See "Memory and the Cap on Files in Flight" in `sec/design/pipeline/ingestion_paths.md`.
pub const DEFAULT_FILES_IN_FLIGHT: usize = 2;
