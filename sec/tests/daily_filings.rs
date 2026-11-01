//! # `daily_filings` Tests
//!
//! Runs the tests of the modules of the `daily_filings` binary. The binary is its own crate root,
//! so this test compiles its modules as private modules, as `notes_backfill.rs` does.

#[path = "../src/bin/daily_filings/constants.rs"]
mod constants;
#[path = "../src/bin/daily_filings/dates.rs"]
mod dates;
#[path = "../src/bin/daily_filings/run.rs"]
mod run;
